use std::{
    fmt::Debug,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use timecode::{
    framerates::*, ConstFramerate, Convert, DynFramerate, Frames, Timecode, ToFrames,
    ValidateableFramerate,
};

fn test_sdvi_frame_count<FR, P>(path: P)
where
    FR: ValidateableFramerate + Debug + Eq + ConstFramerate,
    P: AsRef<Path>,
{
    let f = BufReader::new(File::open(path).unwrap());

    for line in f.lines().map(|x| x.unwrap()) {
        let parts: Vec<_> = line.split("|").collect();

        let my_version: Timecode<FR> =
            Timecode::from_frames(&Frames(parts[0].parse().unwrap()), &FR::new());
        let sdvi_version: Timecode<FR> = parts[1].parse().unwrap();

        dbg!("{} {}", my_version, sdvi_version);

        assert_eq!(
            my_version, sdvi_version,
            "lib,sdvi : {},{}",
            my_version, sdvi_version
        );
    }
}

fn test_sdvi_frame_convert<FRS, FRD, P>(path: P, fr_src: &FRS, fr_dst: &FRD)
where
    FRS: ValidateableFramerate + Debug + Eq,
    FRD: ValidateableFramerate + Debug + Eq,
    P: AsRef<Path>,
{
    let f = BufReader::new(File::open(path).unwrap());
    for line in f.lines().map(|x| x.unwrap()) {
        let parts: Vec<_> = line.split("|").collect();

        let my_version = Timecode::from_frames(&Frames(parts[0].parse().unwrap()), fr_src);
        let my_version = my_version.convert_with_fr(fr_dst);
        let sdvi_version = timecode::unvalidated(parts[1])
            .unwrap()
            .validate_with_fr(fr_dst)
            .unwrap();

        let difference = my_version.to_frame_count() as i64 - sdvi_version.to_frame_count() as i64;

        assert!(difference.abs() <= 1, "{}", parts[0]);
    }
}

#[test]
fn test_sdvi_frame_count_2997() {
    test_sdvi_frame_count::<DF<30>, _>("./tests/sdvi.txt");
}

#[test]
fn test_sdvi_frame_count_25() {
    test_sdvi_frame_count::<NDF<25>, _>("./tests/sdvi_25.txt");
}

#[test]
fn test_sdvi_frame_count_5994() {
    test_sdvi_frame_count::<DF<60>, _>("./tests/sdvi_5994.txt");
}

#[test]
fn test_sdvi_frame_count_convert_25_2997() {
    test_sdvi_frame_convert("./tests/sdvi_convert.txt", &NDF::<25>, &DF::<30>);
}

#[test]
fn test_sdvi_frame_count_convert_2997_25() {
    test_sdvi_frame_convert("./tests/sdvi_convert_rev.txt", &DF::<30>, &NDF::<25>);
}

#[test]
fn test_sdvi_frame_count_convert_50_25() {
    test_sdvi_frame_convert("./tests/sdvi_convert_50.txt", &NDF::<50>, &NDF::<25>);
}

#[test]
fn test_sdvi_frame_count_convert_25_2997_dyn() {
    let a: DynFramerate = "25".parse().unwrap();
    let b: DynFramerate = "29.97".parse().unwrap();
    test_sdvi_frame_convert("./tests/sdvi_convert.txt", &a, &b);
}

#[test]
fn test_sdvi_frame_count_convert_2997_25_dyn() {
    let a: DynFramerate = "29.97".parse().unwrap();
    let b: DynFramerate = "25".parse().unwrap();
    test_sdvi_frame_convert("./tests/sdvi_convert_rev.txt", &a, &b);
}

#[test]
fn test_sdvi_frame_count_convert_50_25_dyn() {
    let a: DynFramerate = "50".parse().unwrap();
    let b: DynFramerate = "25".parse().unwrap();
    test_sdvi_frame_convert("./tests/sdvi_convert_50.txt", &a, &b);
}
