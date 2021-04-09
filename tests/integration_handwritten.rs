use std::fmt::Write;

use smallstr::SmallString;
use timecode::{Timecode, framerates::*, Frames, ValidateableFramerate, ToFrames};

fn add_single_frame<FR: ValidateableFramerate>(input: &str, expected: &str) {
    let tc: Timecode<FR> = input.parse().unwrap();
    let tc = tc + Frames(1);

    let mut b = SmallString::<[u8; 14]>::new();

    write!(b, "{}", tc).unwrap();

    assert_eq!(expected, b.as_str(), "expected left got right");
}

fn test_framecount<FR: ValidateableFramerate>(input: &str, expected_count: u32) {
    let tc: Timecode<FR> = input.parse().unwrap();
    let count = tc.to_frame_count();

    assert!(count == expected_count, "input: {} expected {} frames but got {}", input, expected_count, count);
}

#[test]
fn test_add_ndf() {
    add_single_frame::<NDF30>("00:01:02:00", "00:01:02:01");
    add_single_frame::<NDF30>("00:01:02:29", "00:01:03:00");
    add_single_frame::<NDF30>("00:01:59:29", "00:02:00:00");
    add_single_frame::<NDF30>("00:59:59:29", "01:00:00:00");
}

#[test]
fn test_add_df() {
    add_single_frame::<DF2997>("00:01:02;00", "00:01:02;01");
    add_single_frame::<DF2997>("00:08:59;29", "00:09:00;02");
    add_single_frame::<DF2997>("00:09:59;29", "00:10:00;00");
}

#[test]
fn test_to_frames_for_ndf() {
    test_framecount::<NDF30>("00:01:02:00", 1860);
    test_framecount::<NDF30>("00:01:02:29", 1860 + 29);
    test_framecount::<NDF30>("00:01:59:29", 1860 + 29 + 57 * 30);
    test_framecount::<NDF30>("00:59:59:29", 59*60*30 + 59*30 + 29);
}

#[test]
fn test_to_frames_for_df() {
    test_framecount::<DF2997>("00:00:00;01", 1);
    test_framecount::<DF2997>("00:09:00;02", 16184);
    test_framecount::<DF2997>("00:08:59;29", 16183);
}

#[test]
fn test_reversable() {
    for count in 0u32..(60 * 60 * 30 * 24) {
        //let mut b = SmallString::<[u8; 14]>::new();

        let tc: Timecode<DF2997> = Timecode::from_frames(&Frames(count));
        assert_eq!(tc.to_frame_count(), count);

        let tc: Timecode<NDF30> = Timecode::from_frames(&Frames(count));
        assert_eq!(tc.to_frame_count(), count);
    }
}

#[test]
fn test_convert() {
    let tc: Timecode<NDF30> = "01:00:00:00".parse().unwrap();
    let tc2 = tc.convert_to::<DF2997>();

    let mut b = SmallString::<[u8; 14]>::new();

    write!(b, "{}", tc2).unwrap();

    assert_eq!("01:00:00;00", b);
}
