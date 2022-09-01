use std::fmt::Write;

use smallstr::SmallString;
use timecode::{
    framerates::*, ConstFramerate, Convert, Frames, Timecode, ToFrames, ValidateableFramerate,
};

fn add_single_frame<FR: ValidateableFramerate + ConstFramerate>(input: &str, expected: &str) {
    let tc: Timecode<FR> = input.parse().unwrap();
    let tc = tc + Frames(1);

    let mut b = SmallString::<[u8; 14]>::new();

    write!(b, "{}", tc).unwrap();

    assert_eq!(expected, b.as_str(), "expected left got right");
}

fn test_framecount<FR: ValidateableFramerate + ConstFramerate>(input: &str, expected_count: u32) {
    let tc: Timecode<FR> = input.parse().unwrap();
    let count = tc.to_frame_count();

    assert!(
        count == expected_count,
        "input: {} expected {} frames but got {}",
        input,
        expected_count,
        count
    );
}

#[test]
fn test_add_ndf() {
    add_single_frame::<NDF<30>>("00:01:02:00", "00:01:02:01");
    add_single_frame::<NDF<30>>("00:01:02:29", "00:01:03:00");
    add_single_frame::<NDF<30>>("00:01:59:29", "00:02:00:00");
    add_single_frame::<NDF<30>>("00:59:59:29", "01:00:00:00");
}

#[test]
fn test_add_df() {
    add_single_frame::<DF<30>>("00:01:02;00", "00:01:02;01");
    add_single_frame::<DF<30>>("00:08:59;29", "00:09:00;02");
    add_single_frame::<DF<30>>("00:09:59;29", "00:10:00;00");
}

#[test]
fn test_to_frames_for_ndf() {
    test_framecount::<NDF<30>>("00:01:02:00", 1860);
    test_framecount::<NDF<30>>("00:01:02:29", 1860 + 29);
    test_framecount::<NDF<30>>("00:01:59:29", 1860 + 29 + 57 * 30);
    test_framecount::<NDF<30>>("00:59:59:29", 59 * 60 * 30 + 59 * 30 + 29);
}

#[test]
fn test_to_frames_for_df() {
    test_framecount::<DF<30>>("00:00:00;01", 1);
    test_framecount::<DF<30>>("00:09:00;02", 16184);
    test_framecount::<DF<30>>("00:08:59;29", 16183);
}

#[test]
fn test_reversable() {
    for count in 0u32..(60 * 60 * 30 * 24) {
        //let mut b = SmallString::<[u8; 14]>::new();

        let tc: Timecode<DF<30>> = Timecode::from_frames(&Frames(count), &DF::<30>);
        assert_eq!(tc.to_frame_count(), count);

        let tc: Timecode<NDF<30>> = Timecode::from_frames(&Frames(count), &NDF::<30>);
        assert_eq!(tc.to_frame_count(), count);
    }
}

#[test]
fn test_convert() {
    let tc: Timecode<NDF<30>> = "01:00:00:00".parse().unwrap();
    let tc2 = tc.convert::<DF<30>>();

    let mut b = SmallString::<[u8; 14]>::new();

    write!(b, "{}", tc2).unwrap();

    assert_eq!(&b, "01:00:00;00");
}

#[test]
fn test_convert_start() {
    let tc: Timecode<NDF<30>> = "01:00:00:00".parse().unwrap();
    let tc2 = tc.convert_with_start::<DF<30>>("01:00:00:00".parse().unwrap());

    let mut b = SmallString::<[u8; 14]>::new();

    write!(b, "{}", tc2).unwrap();

    assert_eq!(&b, "01:00:00;00");
}

#[test]
fn test_convert_symmetry_5994() {
    let bads = [3597, 5395, 7193, 17981, 19781];
    let near_bad = bads
        .iter()
        .map(|x| ((-100)..100).map(move |n| ((*x as i32) + n) as u32))
        .flatten();

    for i in near_bad {
        let input = Timecode::from_frames(&Frames(i), &DF::<30>);
        let output: Timecode<DF<60>> = input.convert();
        let incheck = output.convert();
        dbg!(i);
        assert_eq!(input, incheck);
    }
}
