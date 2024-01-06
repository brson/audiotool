use rx::prelude::*;
use crate::types::{Format, BitDepth, SampleRate, Codec};
use crate::io::{PcmReader, PcmWriter, Buf, Props};
use std::path::Path;
use std::io::{BufReader, BufWriter};
use std::fs::File;
use std::ptr::NonNull;
use std::ffi::{c_void, CStr};
use std::ffi::CString;
use libflac_sys::*;
use rx::libc::c_char;

pub struct FlacPcmReader {
    decoder: AnyResult<NonNull<FLAC__StreamDecoder>>,
    cbdata: *mut ReaderCallbackData,
}

struct ReaderCallbackData {
    props: Option<Props>,
    buf: Buf,
    error: AnyResult<()>,
}

unsafe impl Send for FlacPcmReader { }

impl FlacPcmReader {
    pub fn new(path: &Path) -> FlacPcmReader {
        let mut cbdata = Box::new(ReaderCallbackData {
            props: None,
            buf: Buf::Uninit,
            error: Ok(()),
        });

        unsafe {
            let decoder = FLAC__stream_decoder_new();
            let decoder = NonNull::new(decoder);
            let decoder = decoder.ok_or_else(|| {
                anyhow!("unable to allocate FLAC decoder")
            });

            let decoder = if let Ok(decoder) = decoder {
                FLAC__stream_decoder_set_md5_checking(decoder.as_ptr(), true as FLAC__bool);

                let path = path.to_str().expect("todo utf8 path").to_owned();
                let path = CString::new(path).expect("path with nul bytes").to_owned();

                let status = FLAC__stream_decoder_init_file(
                    decoder.as_ptr(),
                    path.as_ptr(),
                    Some(decoder_write_callback),
                    Some(decoder_metadata_callback),
                    Some(decoder_error_callback),
                    cbdata.as_mut() as *mut ReaderCallbackData as *mut c_void,
                );

                if status == FLAC__STREAM_DECODER_INIT_STATUS_OK {
                    Ok(decoder)
                } else {
                    FLAC__stream_decoder_delete(decoder.as_ptr());
                    let err_str = code_to_string(&FLAC__StreamDecoderInitStatusString, status);
                    Err(anyhow!("{err_str}"))
                }
            } else {
                decoder
            };

            FlacPcmReader {
                decoder,
                cbdata: Box::leak(cbdata) as *mut ReaderCallbackData,
            }
        }
    }
}

extern "C" fn decoder_write_callback(
    decoder: *const FLAC__StreamDecoder,
    frame: *const FLAC__Frame,
    buffer: *const *const i32,
    cbdata: *mut c_void,
) -> FLAC__StreamDecoderWriteStatus {
    assert!(!decoder.is_null());
    assert!(!frame.is_null());
    assert!(!buffer.is_null());
    assert!(!cbdata.is_null());

    unsafe {
        let cbdata = &mut *(cbdata as *mut ReaderCallbackData);

        // Can't support properties changing between frames.
        if let Some(props) = cbdata.props {
            assert_eq!(
                props.format.sample_rate.as_u32(),
                (*frame).header.sample_rate,
            );
            match props.format.bit_depth {
                BitDepth::F32 => unreachable!(),
                BitDepth::I24 => assert_eq!(
                    24, (*frame).header.bits_per_sample,
                ),
                BitDepth::I16 => assert_eq!(
                    16, (*frame).header.bits_per_sample,
                ),
            }
            assert_eq!(
                props.channels as u32,
                (*frame).header.channels,
            );
        }

        match (*frame).header.bits_per_sample {
            24 => {
                let buf = cbdata.buf.i24_mut();

                // Interleave channels from individual buffers
                for block in 0..(*frame).header.blocksize as isize {
                    for ch in 0..(*frame).header.channels as isize {
                        let channel_buf = *buffer.offset(ch);
                        let sample = *channel_buf.offset(block);
                        buf.push(sample);
                    }
                }
            }
            16 => {
                let buf = cbdata.buf.i16_mut();

                // Interleave channels from individual buffers
                for block in 0..(*frame).header.blocksize as isize {
                    for ch in 0..(*frame).header.channels as isize {
                        let channel_buf = *buffer.offset(ch);
                        let sample = *channel_buf.offset(block);
                        buf.push(sample as i16);
                    }
                }
            }
            v => todo!("flac bits per sample {v}"),
        }
    }

    FLAC__STREAM_DECODER_WRITE_STATUS_CONTINUE
}

extern "C" fn decoder_metadata_callback(
    decoder: *const FLAC__StreamDecoder,
    metadata: *const FLAC__StreamMetadata,
    cbdata: *mut c_void,
) {
    assert!(!decoder.is_null());
    assert!(!metadata.is_null());
    assert!(!cbdata.is_null());

    unsafe {
        let cbdata = &mut *(cbdata as *mut ReaderCallbackData);

        if (*metadata).type_ == FLAC__METADATA_TYPE_STREAMINFO {
            let stream_info = &(*metadata).data.stream_info;

            let bit_depth = match stream_info.bits_per_sample {
                24 => BitDepth::I24,
                16 => BitDepth::I16,
                v => todo!("flac bits per sample {v}"),
            };

            let sample_rate = match stream_info.sample_rate {
                192_000 => SampleRate::K192,
                48_000 => SampleRate::K48,
                v => todo!("flac sample rate {v}"),
            };

            let channels = match stream_info.channels {
                1 => 1,
                2 => 2,
                v => todo!("flac channels {v}"),
            };

            let props = Props {
                channels,
                format: Format {
                    codec: Codec::Flac,
                    bit_depth,
                    sample_rate,
                }
            };

            assert!(cbdata.props.is_none());

            cbdata.props = Some(props);
        }
    }
}

extern "C" fn decoder_error_callback(
    decoder: *const FLAC__StreamDecoder,
    status: FLAC__StreamDecoderErrorStatus,
    cbdata: *mut c_void,
) {
    assert!(!decoder.is_null());
    assert!(!cbdata.is_null());

    unsafe {
        let cbdata = &mut *(cbdata as *mut ReaderCallbackData);

        let err_str = code_to_string(&FLAC__StreamDecoderErrorStatusString, status);
        cbdata.error = Err(anyhow!("{err_str}"));
    }
}

impl Drop for FlacPcmReader {
    fn drop(&mut self) {
        unsafe {
            if let Ok(decoder) = self.decoder.as_ref() {
                FLAC__stream_decoder_delete(decoder.as_ptr());
            }

            let _cbdata = Box::from_raw(self.cbdata);
        }
    }
}

impl PcmReader for FlacPcmReader {
    fn props(&mut self) -> AnyResult<Props> {
        let decoder = self.decoder.as_ref()
            .map_err(|e| anyhow!("{e}"))?;

        unsafe {
            // Take and drop references to the shared cbdata
            // before calling the decoder, which will mutate them.
            {
                let error = &(*self.cbdata).error;

                if let Err(e) = error {
                    bail!("{e}");
                }

                let props = &(*self.cbdata).props;

                if let Some(props) = props {
                    return Ok(*props);
                }
            };

            let ok = FLAC__stream_decoder_process_until_end_of_metadata(decoder.as_ptr()) != 0;

            if !ok {
                if let Err(e) = &(*self.cbdata).error {
                    bail!("{e}");
                } else {
                    let state = FLAC__stream_decoder_get_state(decoder.as_ptr());
                    let err_str = code_to_string(&FLAC__StreamDecoderStateString, state);
                    return Err(anyhow!("{err_str}"));
                }
            }

            assert!((*self.cbdata).props.is_some());
            self.props()
        }
    }

    fn read(
        &mut self,
        buf: &mut Buf,
    ) -> AnyResult<()> {
        let decoder = self.decoder.as_ref()
            .map_err(|e| anyhow!("{e}"))?;

        buf.truncate();

        unsafe {
            loop {
                // Take and drop references to the shared cbdata
                // before calling the decoder, which will mutate them.
                {
                    let error = &(*self.cbdata).error;

                    if let Err(e) = error {
                        bail!("{e}");
                    }

                    let self_buf = &mut (*self.cbdata).buf;

                    if !self_buf.is_empty() {
                        std::mem::swap(self_buf, buf);
                        assert!(self_buf.is_empty());
                        return Ok(());
                    }
                }

                let state = FLAC__stream_decoder_get_state(decoder.as_ptr());
                if state == FLAC__STREAM_DECODER_END_OF_STREAM {
                    return Ok(());
                }
                
                let ok = FLAC__stream_decoder_process_single(decoder.as_ptr());

                if ok == 0 {
                    if let Err(e) = &(*self.cbdata).error {
                        bail!("{e}");
                    } else {
                        let state = FLAC__stream_decoder_get_state(decoder.as_ptr());
                        let err_str = code_to_string(&FLAC__StreamDecoderStateString, state);
                        return Err(anyhow!("{err_str}"));
                    }
                }
            }
        }
    }
}

pub struct FlacPcmWriter {
    encoder: AnyResult<NonNull<FLAC__StreamEncoder>>,
    props: Props,
}

unsafe impl Send for FlacPcmWriter { }

impl FlacPcmWriter {
    pub fn new(
        path: &Path,
        props: Props,
    ) -> FlacPcmWriter {
        assert_eq!(props.format.codec, Codec::Flac);

        unsafe {
            let encoder = FLAC__stream_encoder_new();
            let encoder = NonNull::new(encoder);
            let encoder = encoder.ok_or_else(|| {
                anyhow!("unable to allocate FLAC encoder")
            });

            let bits_per_sample = match props.format.bit_depth {
                BitDepth::F32 => unreachable!(),
                BitDepth::I24 => 24,
                BitDepth::I16 => 16,
            };

            let encoder = if let Ok(encoder) = encoder {
                let ok = {
	                //FLAC__stream_encoder_set_verify(encoder.as_ptr(), true as FLAC__bool) != 0
                    // fixme don't hardcode 5
	                FLAC__stream_encoder_set_compression_level(encoder.as_ptr(), 5) != 0
	                    && FLAC__stream_encoder_set_channels(encoder.as_ptr(), props.channels as u32) != 0
	                    && FLAC__stream_encoder_set_bits_per_sample(encoder.as_ptr(), bits_per_sample) != 0
	                    && FLAC__stream_encoder_set_sample_rate(encoder.as_ptr(), props.format.sample_rate.as_u32()) != 0
                    // todo
                    //FLAC__stream_encoder_set_total_samples_estimate(encoder, total_samples);
                };

                if ok {
                    Ok(encoder)
                } else {
                    let state = FLAC__stream_encoder_get_state(encoder.as_ptr());
                    let err_str = code_to_string(&FLAC__StreamEncoderStateString, state);
                    FLAC__stream_encoder_delete(encoder.as_ptr());
                    Err(anyhow!("{err_str}"))
                }
            } else {
                encoder
            };

            let encoder = if let Ok(encoder) = encoder {

                let path = path.to_str().expect("todo utf8 path").to_owned();
                let path = CString::new(path).expect("path with nul bytes").to_owned();

                let status = FLAC__stream_encoder_init_file(
                    encoder.as_ptr(),
                    path.as_ptr(),
                    None,
                    std::ptr::null_mut(),
                );

                if status == FLAC__STREAM_ENCODER_INIT_STATUS_OK {
                    Ok(encoder)
                } else {
                    FLAC__stream_encoder_delete(encoder.as_ptr());
                    let err_str = code_to_string(&FLAC__StreamEncoderInitStatusString, status);
                    Err(anyhow!("{err_str}"))
                }
            } else {
                encoder
            };

            FlacPcmWriter {
                encoder,
                props,
            }
        }
    }
}

impl Drop for FlacPcmWriter {
    fn drop(&mut self) {
        unsafe {
            if let Ok(encoder) = self.encoder.as_ref() {
                FLAC__stream_encoder_delete(encoder.as_ptr());
            }
        }
    }
}

impl PcmWriter for FlacPcmWriter {
    fn write(
        &mut self,
        buf: &Buf,
    ) -> AnyResult<()> {
        let encoder = self.encoder.as_ref()
            .map_err(|e| anyhow!("{e}"))?;

        assert_eq!(buf.bit_depth(), Some(self.props.format.bit_depth));

        unsafe {
            let mut tmp_buf = Vec::<i32>::new();
            let samples = match buf {
                Buf::Uninit => unreachable!(),
                Buf::F32(_) => unreachable!(),
                Buf::I24(buf) => &buf,
                Buf::I16(buf) => {
                    tmp_buf = buf.iter().map(|s| *s as i32).collect();
                    &tmp_buf
                }
            };

            assert_eq!(samples.len() % self.props.channels as usize, 0);
            let samples_len = samples.len() / self.props.channels as usize;

            let ok = FLAC__stream_encoder_process_interleaved(
                encoder.as_ptr(),
                samples.as_ptr(),
                samples_len as u32,
            ) != 0;

            if ok {
                Ok(())
            } else {
                let state = FLAC__stream_encoder_get_state(encoder.as_ptr());
                let err_str = code_to_string(&FLAC__StreamEncoderStateString, state);
                Err(anyhow!("{err_str}"))
            }
        }
    }

    fn finalize(&mut self) -> AnyResult<()> {
        let encoder = self.encoder.as_ref()
            .map_err(|e| anyhow!("{e}"))?;

        unsafe {
            let ok = FLAC__stream_encoder_finish(
                encoder.as_ptr(),
            ) != 0;

            if ok {
                Ok(())
            } else {
                let state = FLAC__stream_encoder_get_state(encoder.as_ptr());
                let err_str = code_to_string(&FLAC__StreamEncoderStateString, state);
                Err(anyhow!("{err_str}"))
            }
        }
    }
}

unsafe fn code_to_string(
    table: &[*const c_char; 0],
    code: u32,
) -> String {
    let cstr_ptr = table.as_ptr().offset(code as isize);
    let cstr = CStr::from_ptr(*cstr_ptr);
    cstr.to_str().expect("utf8").to_owned()
}
