# Encodings

maybe just use libavcodec / libavformat

- WAV
  - crate: hound
- FLAC
  - lib: xiph/flac
  - containers: flac
  - meta: ogg comments
- ALAC
  - lib: macosforge/alac
  - container: m4a
- Vorbis
  - lib: xiph/vorbis
  - containers: ogg
  - meta: ogg comments
- MP3
  - lib: libmp3lame / libavcodec
  - container: mp3
  - meta: id3
- AAC
  - lib: libavcodec
    - with libfdk_aac
  - container: m4a

# Containers

- FLAC
  - lib: xiph/flac
- MP4 (M4A)
- Ogg
  - lib: xiph/ogg
  - crate: rustaudio/ogg
- MP3
  - lib: lame

# Metadata

- FLAC / Ogg
- ID3
- mp4

# Other tools

- Sample rate conversion (libsrc)
- Dithering (?)

# Dithering

- triangular probability density function (tpdf)
- airwindows
- https://gearspace.com/board/showpost.php?p=12360487&postcount=14
- https://gearspace.com/gear/airwindows/tpdf?via=gear_post_link
