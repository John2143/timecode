use std::{
    fmt::{Debug, Write},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use smallstr::SmallString;
use timecode::{framerates::*, Framerate, Frames, Timecode, ToFrames, ValidateableFramerate};

fn test_sdvi_frame_count<FR: ValidateableFramerate + Debug + Eq, P: AsRef<Path>>(path: P) {
    let f = BufReader::new(File::open(path).unwrap());

    for line in f.lines().map(|x| x.unwrap()) {
        let parts: Vec<_> = line.split("|").collect();

        let my_version: Timecode<FR> = Timecode::from_frames(&Frames(parts[0].parse().unwrap()));
        let sdvi_version: Timecode<FR> = parts[1].parse().unwrap();

        assert_eq!(my_version, sdvi_version);
    }
}

fn test_sdvi_frame_convert<FRS, FRD, P>(path: P)
where
    FRS: ValidateableFramerate + Debug + Eq,
    FRD: ValidateableFramerate + Debug + Eq,
    P: AsRef<Path>,
{
    let f = BufReader::new(File::open(path).unwrap());
    for line in f.lines().map(|x| x.unwrap()) {
        let parts: Vec<_> = line.split("|").collect();

        let my_version: Timecode<FRS> = Timecode::from_frames(&Frames(parts[0].parse().unwrap()));
        let my_version: Timecode<FRD> = my_version.convert_to();

        let sdvi_version: Timecode<FRD> = parts[1].parse().unwrap();

        let difference = my_version.to_frame_count() as i64 - sdvi_version.to_frame_count() as i64;

        assert!(difference.abs() <= 1, "{}", parts[0]);
    }
}

#[test]
fn test_sdvi_frame_count_2997() {
    test_sdvi_frame_count::<DF2997, _>("./tests/sdvi.txt");
}

#[test]
fn test_sdvi_frame_count_25() {
    test_sdvi_frame_count::<NDF25, _>("./tests/sdvi_25.txt");
}

#[test]
fn test_sdvi_frame_count_convert_25_2997() {
    test_sdvi_frame_convert::<NDF25, DF2997, _>("./tests/sdvi_convert.txt");
}

#[test]
fn test_sdvi_frame_count_convert_2997_25() {
    test_sdvi_frame_convert::<DF2997, NDF25, _>("./tests/sdvi_convert_rev.txt");
}

#[test]
fn test_sdvi_frame_count_convert_50_25() {
    test_sdvi_frame_convert::<NDF50, NDF25, _>("./tests/sdvi_convert_50.txt");
}
