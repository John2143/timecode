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

fn test_reference_frame_count<FR, P>(path: P, fr: &FR)
where
    FR: ValidateableFramerate + Debug + Eq + ConstFramerate,
    P: AsRef<Path>,
{
    let f = BufReader::new(File::open(path).unwrap());

    for line in f.lines().map(|x| x.unwrap()) {
        let parts: Vec<_> = line.split("|").collect();

        let my_version = Timecode::from_frames(&Frames(parts[0].parse().unwrap()), fr);
        let reference_version: Timecode<FR> = parts[1].parse().unwrap();

        dbg!("{} {}", my_version, reference_version);

        assert_eq!(
            my_version, reference_version,
            "lib,reference : {},{}",
            my_version, reference_version
        );
    }
}

fn test_reference_frame_convert<FRS, FRD, P>(path: P, fr_src: &FRS, fr_dst: &FRD)
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
        let reference_version = timecode::unvalidated(parts[1])
            .unwrap()
            .validate_with_fr(fr_dst)
            .unwrap();

        let difference =
            my_version.to_frame_count() as i64 - reference_version.to_frame_count() as i64;

        assert!(difference.abs() <= 1, "{}", parts[0]);
    }
}

#[test]
fn test_reference_frame_count_2997() {
    test_reference_frame_count("./tests/samples/reference.txt", &DF2997::new());
}

#[test]
fn test_reference_frame_count_25() {
    test_reference_frame_count("./tests/samples/reference_25.txt", &NDF25::new());
}

#[test]
fn test_reference_frame_count_5994() {
    test_reference_frame_count("./tests/samples/reference_5994.txt", &DF5994::new());
}

#[test]
fn test_reference_frame_count_convert_25_2997() {
    test_reference_frame_convert(
        "./tests/samples/reference_convert.txt",
        &NDF::<25>,
        &DF::<30>,
    );
}

#[test]
fn test_reference_frame_count_convert_2997_25() {
    test_reference_frame_convert(
        "./tests/samples/reference_convert_rev.txt",
        &DF::<30>,
        &NDF::<25>,
    );
}

#[test]
fn test_reference_frame_count_convert_50_25() {
    test_reference_frame_convert(
        "./tests/samples/reference_convert_50.txt",
        &NDF::<50>,
        &NDF::<25>,
    );
}

#[test]
fn test_reference_frame_count_convert_25_2997_dyn() {
    let a: DynFramerate = "25".parse().unwrap();
    let b: DynFramerate = "29.97".parse().unwrap();
    test_reference_frame_convert("./tests/samples/reference_convert.txt", &a, &b);
}

#[test]
fn test_reference_frame_count_convert_2997_25_dyn() {
    let a: DynFramerate = "29.97".parse().unwrap();
    let b: DynFramerate = "25".parse().unwrap();
    test_reference_frame_convert("./tests/samples/reference_convert_rev.txt", &a, &b);
}

#[test]
fn test_reference_frame_count_convert_50_25_dyn() {
    let a: DynFramerate = "50".parse().unwrap();
    let b: DynFramerate = "25".parse().unwrap();
    test_reference_frame_convert("./tests/samples/reference_convert_50.txt", &a, &b);
}
