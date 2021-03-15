use std::fmt::Write;

use smallstr::SmallString;
use timecode::{Timecode, framerates::*, Frames, ValidateableFramerate};

fn add_single_frame<FR: ValidateableFramerate>(input: &str, expected: &str) {
    let tc: Timecode<FR> = input.parse().unwrap();
    let tc = tc + Frames(1);

    let mut b = SmallString::<[u8; 14]>::new();

    write!(b, "{}", tc).unwrap();

    assert_eq!(expected, b.as_str());
}

#[test]
fn test_add() {
    add_single_frame::<NDF30>("00:01:02:00", "00:01:02:01");
    add_single_frame::<NDF30>("00:01:02:29", "00:01:03:00");
}
