use rx::proptest::prelude::*;
use audiotool::bitdepth::*;

fn do_i16_to_f32_roundtrip(i1: i16) {
    let f = i16_to_f32(i1);
    let i2 = f32_to_i16(f);
    assert_eq!(i1, i2);
}

#[test]
fn i16_to_f32_roundtrips() {
    do_i16_to_f32_roundtrip(0);
    do_i16_to_f32_roundtrip(i16::MIN);
    do_i16_to_f32_roundtrip(i16::MAX);
}

proptest! {
    #[test]
    fn i16_to_f32_roundtrip(
        i1 in any::<i16>()
    ) {
        do_i16_to_f32_roundtrip(i1);
    }
}
