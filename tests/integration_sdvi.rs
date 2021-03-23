use std::fmt::Write;

use smallstr::SmallString;
use timecode::{Timecode, framerates::*, Frames, ValidateableFramerate, ToFrames};

fn add_single_frame<FR: ValidateableFramerate>(input: &str, expected: &str) {
    let tc: Timecode<FR> = input.parse().unwrap();
    let tc = tc + Frames(1);

    let mut b = SmallString::<[u8; 14]>::new();

    write!(b, "{}", tc).unwrap();

    assert_eq!(expected, b.as_str());
}

fn test_framecount<FR: ValidateableFramerate>(input: &str, expected_count: usize) {
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
    //add_single_frame::<DF2997>("00:01:02:00", "00:01:02:01");
    //add_single_frame::<DF2997>("00:01:02:29", "00:01:03:02");
    //add_single_frame::<DF2997>("00:01:59:29", "00:02:00:00");
    //add_single_frame::<DF2997>("00:59:59:29", "01:00:00:00");
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
    //let count_959 = "00:09:59:29".parse::<Timecode<DF2997>>().unwrap().to_frame_count();
    //test_framecount::<DF2997>("00:09:59:29", count_959);
    //test_framecount::<DF2997>("00:10:", );
    //test_framecount::<DF2997>("00:01:59:29", 1860 + 29 + 57 * 30);
    //test_framecount::<DF2997>("00:59:59:29", 59*60*30 + 59*30 + 29);
}