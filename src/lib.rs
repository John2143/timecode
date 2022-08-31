#![allow(dead_code)]
//!This is a correct implementation of SMPTE timecodes used at
//![`Discovery`](https://github.com/discoveryinc-cs)
//!
//!# Quickstart
//!
//!The fastest way to get started is to parse a timecode directly with [`str::parse`](std::primitive::str::parse).
//!
//!```
//!use timecode::{framerates::*, Timecode};
//!
//!let tc: Timecode<NDF<30>> = "01:02:00:25".parse().expect("Couldn't convert to NDF30 timecode");
//!
//!assert_eq!(tc.h(), 1);
//!assert_eq!(tc.m(), 2);
//!assert_eq!(tc.s(), 0);
//!assert_eq!(tc.f(), 25);
//!assert_eq!(tc.to_string(), "01:02:00:25");
//!```
//!
//!If you need more control over the initial parsing, [`unvalidated`] can produce an intermediate
//![`UnvalidatedTC`](parser::UnvalidatedTC) which can be used to create timecodes at multiple
//!different framerates.
//!
//!To access the parsed [`nom`] result directly, see [`parser::timecode_nom`]. [`unvalidated`] is a
//!thin wrapper around [`parser::timecode_nom`] which fails on remaining input and hides the
//!parsing error.
//!
//!
//!```
//!use timecode::framerates::*;
//!//Parse a string into an Option<UnvalidatedTC>
//!let raw_tc = timecode::unvalidated("01:02:00:25").unwrap();
//!
//!//Call validate with your desired framerate to get a Result<Timecode>
//!let tc = raw_tc.validate::<NDF<30>>().unwrap();
//!
//!assert_eq!(tc.to_string(), "01:02:00:25");
//!assert_eq!(tc.h(), 1);
//!assert_eq!(tc.m(), 2);
//!assert_eq!(tc.s(), 0);
//!assert_eq!(tc.f(), 25);
//!
//!//01:02:00:25 is not a valid 2398 timecode.
//!let invalid_tc = raw_tc.validate::<NDF<24>>();
//!assert!(invalid_tc.is_err());
//!
//!//Dropframe invariants are also checked.
//!let invalid_tc = timecode::unvalidated("01:02:00;01").unwrap().validate::<DF2997>();
//!assert!(invalid_tc.is_err());
//!```

use std::{convert::TryInto, fmt::Display, str::FromStr};

use parser::UnvalidatedTC;

pub mod parser;
pub mod validate;

#[cfg(feature = "javascript")]
pub mod javascript;
#[cfg(feature = "python")]
pub mod python;

pub use parser::unvalidated;

macro_rules! framerate_impl {
    ($i: ident = $rep: expr, $sep: expr, $max_frame: expr, df = $is_dropframe: expr, $fr_num: expr ; $fr_den: expr, ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $i;

        impl crate::ConstFramerate for $i {
            fn new() -> Self {
                $i
            }
        }

        impl crate::Framerate for $i {
            fn to_sep(&self) -> char {
                $sep
            }

            fn max_frame(&self) -> crate::FrameCount {
                $max_frame
            }

            fn is_dropframe(&self) -> bool {
                $is_dropframe
            }

            fn fr_ratio(&self) -> f32 {
                self.fr_num() as f32 / self.fr_denom() as f32
            }

            fn fr_num(&self) -> u64 {
                $fr_num
            }

            fn fr_denom(&self) -> u64 {
                $fr_den
            }
        }
    };
}

pub mod framerates {
    use std::convert::TryFrom;

    framerate_impl! {
        DF2997 = "29.97",
        ';', 30, df = true,
        30000 ; 1001,
    }

    framerate_impl! {
        DF5994 = "59.94",
        ';', 30, df = true,
        30000 ; 1001,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct NDFDyn(pub crate::FrameCount);

    impl<const FRAMES: crate::FrameCount> From<NDF<FRAMES>> for NDFDyn {
        fn from(_: NDF<FRAMES>) -> Self {
            Self(FRAMES)
        }
    }

    impl crate::Framerate for NDFDyn {
        fn to_sep(&self) -> char {
            ':'
        }

        fn max_frame(&self) -> crate::FrameCount {
            self.0 //TODO
        }

        fn is_dropframe(&self) -> bool {
            false
        }

        fn fr_ratio(&self) -> f32 {
            self.fr_num() as f32 / self.fr_denom() as f32
        }

        fn fr_num(&self) -> u64 {
            (self.0 as u64) * 1000
        }

        fn fr_denom(&self) -> u64 {
            1000
        }
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct NDF<const FRAMES: crate::FrameCount>;

    impl<const FRAMES: crate::FrameCount> crate::ConstFramerate for NDF<FRAMES> {
        fn new() -> Self {
            Self
        }
    }

    impl<const FRAMES: crate::FrameCount> TryFrom<&NDFDyn> for NDF<FRAMES> {
        type Error = ();

        fn try_from(value: &NDFDyn) -> Result<Self, Self::Error> {
            if value.0 == FRAMES {
                Ok(Self)
            } else {
                Err(())
            }
        }
    }

    impl<const FRAMES: crate::FrameCount> TryFrom<&crate::DynFramerate> for NDF<FRAMES> {
        type Error = ();

        fn try_from(value: &crate::DynFramerate) -> Result<Self, Self::Error> {
            if let &crate::DynFramerate::NDF(n) = value {
                return Self::try_from(&NDFDyn(n));
            }

            Err(())
        }
    }

    impl<const FRAMES: crate::FrameCount> crate::Framerate for NDF<FRAMES> {
        fn to_sep(&self) -> char {
            ':'
        }

        fn max_frame(&self) -> crate::FrameCount {
            FRAMES //TODO
        }

        fn is_dropframe(&self) -> bool {
            false
        }

        fn fr_ratio(&self) -> f32 {
            self.fr_num() as f32 / self.fr_denom() as f32
        }

        fn fr_num(&self) -> u64 {
            (FRAMES as u64) * 1000
        }

        fn fr_denom(&self) -> u64 {
            1000
        }
    }
}

use framerates::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynFramerate {
    DF2997,
    DF5994,
    NDF(FrameCount),
}

impl Framerate for DynFramerate {
    fn to_sep(&self) -> char {
        match self {
            DynFramerate::DF2997 => DF2997.to_sep(),
            DynFramerate::DF5994 => DF5994.to_sep(),
            DynFramerate::NDF(n) => NDFDyn(*n).to_sep(),
        }
    }

    fn max_frame(&self) -> FrameCount {
        match self {
            DynFramerate::DF2997 => DF2997.max_frame(),
            DynFramerate::DF5994 => DF5994.max_frame(),
            DynFramerate::NDF(n) => NDFDyn(*n).max_frame(),
        }
    }

    fn is_dropframe(&self) -> bool {
        match self {
            DynFramerate::DF2997 => DF2997.is_dropframe(),
            DynFramerate::DF5994 => DF5994.is_dropframe(),
            DynFramerate::NDF(n) => NDFDyn(*n).is_dropframe(),
        }
    }

    fn fr_ratio(&self) -> f32 {
        match self {
            DynFramerate::DF2997 => DF2997.fr_ratio(),
            DynFramerate::DF5994 => DF5994.fr_ratio(),
            DynFramerate::NDF(n) => NDFDyn(*n).fr_ratio(),
        }
    }

    fn fr_num(&self) -> u64 {
        match self {
            DynFramerate::DF2997 => DF2997.fr_num(),
            DynFramerate::DF5994 => DF5994.fr_num(),
            DynFramerate::NDF(n) => NDFDyn(*n).fr_num(),
        }
    }

    fn fr_denom(&self) -> u64 {
        match self {
            DynFramerate::DF2997 => DF2997.fr_denom(),
            DynFramerate::DF5994 => DF5994.fr_denom(),
            DynFramerate::NDF(n) => NDFDyn(*n).fr_denom(),
        }
    }
}

impl FromStr for DynFramerate {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DynFramerate::*;

        //if it can be parsed as an integer, assume it is NDF
        if let Ok(fr) = s.parse() {
            return Ok(NDF(fr));
        }

        let special_framerates = [(29.97, DF2997), (23.98, NDF(24)), (59.94, DF5994)];
        if let Ok(float) = s.parse::<f64>() {
            const EPISILON: f64 = 0.01;

            //If it can be parsed as a float, see if it is near a whole number
            if (float - float.round()).abs() < EPISILON {
                return Ok(NDF(float.round() as _));
            }

            //Or if it is a special framerate
            for (fr, s) in special_framerates {
                if (float - fr).abs() < EPISILON {
                    return Ok(s);
                }
            }
        }

        Err("No known dropframe timecode")
    }
}

#[cfg(test)]
mod read_fr_tests {
    use crate::DynFramerate;

    #[test]
    fn read_int() {
        let s: DynFramerate = "25".parse().unwrap();
        assert_eq!(s, DynFramerate::NDF(25));
    }

    #[test]
    fn read_float() {
        let s: DynFramerate = "25.00".parse().unwrap();
        assert_eq!(s, DynFramerate::NDF(25));
    }

    #[test]
    fn read_float_df() {
        let s: DynFramerate = "29.97".parse().unwrap();
        assert_eq!(s, DynFramerate::DF2997);
    }

    #[test]
    fn read_float_ndf_special() {
        let s: DynFramerate = "23.98".parse().unwrap();
        assert_eq!(s, DynFramerate::NDF(24));
    }

    #[test]
    fn read_fr_high() {
        let s: DynFramerate = "239.99".parse().unwrap();
        assert_eq!(s, DynFramerate::NDF(240));
    }
}

pub trait Framerate: Copy {
    fn to_sep(&self) -> char;
    fn max_frame(&self) -> FrameCount;
    fn is_dropframe(&self) -> bool;
    fn fr_ratio(&self) -> f32;
    fn fr_num(&self) -> u64;
    fn fr_denom(&self) -> u64;
}

pub trait ConstFramerate {
    fn new() -> Self;
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TimecodeValidationError {
    ///The minutes field is invalid
    InvalidMin,
    ///The seconds field is invalid
    InvalidSec,
    ///The frames field is invalid (can happen because target is drop-frame)
    InvalidFrames,
    ///This is the error received when nom fails to parse the timecode.
    ///This will never occur when you call `.validate`, as by the time you have an unvalidated
    ///timecode to call `.validate` on, it has already passed the parsing step.
    Unparsed,
    //Framerate is bad
    InvalidFramerate,
}

impl ToString for TimecodeValidationError {
    fn to_string(&self) -> String {
        match self {
            TimecodeValidationError::InvalidMin => "Invalid minutes".into(),
            TimecodeValidationError::InvalidSec => "Invalid seconds".into(),
            TimecodeValidationError::InvalidFrames => "Invalid frames".into(),
            TimecodeValidationError::Unparsed => "Timecode cannot be parsed".into(),
            TimecodeValidationError::InvalidFramerate => "Invalid Framerate".into(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TimecodeValidationWarning {
    MismatchSep,
}

///Used internally when calling [`UnvalidatedTC::validate`]. If `Ok(())` is returned, the
///unvalidated timecode will be directly copied into a new [`Timecode`]
pub trait ValidateableFramerate: Framerate + Copy {
    fn validate<T: validate::WarningContainer>(
        &self,
        input_tc: &UnvalidatedTC,
        warnings: &mut T,
    ) -> Result<(), TimecodeValidationError>;
}

//24 hours * 60 * 60 * 120 still has lots of room in a u32
type FrameCount = u32;

#[derive(Copy, Debug, Eq, PartialEq, Clone)]
#[repr(transparent)]
pub struct Frames(pub FrameCount);

#[derive(Copy, Debug, Eq, PartialEq, Clone)]
pub struct Timecode<FR> {
    h: u8,
    m: u8,
    s: u8,
    f: FrameCount,
    framerate: FR,
}

impl<FR: Framerate> Display for Timecode<FR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:02}:{:02}:{:02}{}{:02}",
            self.h,
            self.m,
            self.s,
            self.framerate.to_sep(),
            self.f
        )?;
        Ok(())
    }
}

impl<FR> Timecode<FR> {
    pub fn h(&self) -> u8 {
        self.h
    }
    pub fn m(&self) -> u8 {
        self.m
    }
    pub fn s(&self) -> u8 {
        self.s
    }
    pub fn f(&self) -> FrameCount {
        self.f
    }
    pub fn framerate(&self) -> &FR {
        &self.framerate
    }
}

impl<FR: ValidateableFramerate + ConstFramerate> FromStr for Timecode<FR> {
    type Err = TimecodeValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tc = unvalidated(s).ok_or(TimecodeValidationError::Unparsed)?;

        tc.validate()
    }
}

impl FromStr for Timecode<DynFramerate> {
    type Err = TimecodeValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut at = s.split("@");
        let tc_part = at.next().ok_or(TimecodeValidationError::Unparsed)?;
        let fr_part = at.next().ok_or(TimecodeValidationError::Unparsed)?;
        let tc = unvalidated(tc_part).ok_or(TimecodeValidationError::Unparsed)?;

        let d: DynFramerate = fr_part
            .parse()
            .map_err(|_| TimecodeValidationError::Unparsed)?;

        tc.validate_with_fr(&d)
    }
}

impl Timecode<DynFramerate> {
    fn new_with_fr(s: &str, fr: &str) -> Result<Self, TimecodeValidationError> {
        let tc = unvalidated(s).ok_or(TimecodeValidationError::Unparsed)?;
        let d: DynFramerate = fr
            .parse()
            .map_err(|_| TimecodeValidationError::InvalidFramerate)?;

        tc.validate_with_fr(&d)
    }
}

//Things that can be converted to a frame count
//
//Both [`Timecode`] and [`Frames`] implement this.
pub trait ToFrames<FR> {
    fn to_frame_count(&self) -> FrameCount;
    fn from_frames(f: &Frames, fr: &FR) -> Self;
}

pub trait Convert {
    fn convert<DFR: Framerate + ConstFramerate>(&self) -> Timecode<DFR>;
    fn convert_with_fr<DFR: Framerate>(&self, framerate: &DFR) -> Timecode<DFR>;
    fn convert_with_start<DFR: Framerate + ConstFramerate>(&self, start: Self) -> Timecode<DFR>;
    fn convert_with_start_fr<DFR: Framerate>(&self, start: Self, framerate: &DFR) -> Timecode<DFR>;
}

impl<FR: Framerate> Convert for Timecode<FR> {
    fn convert<DFR: Framerate + ConstFramerate>(&self) -> Timecode<DFR> {
        self.convert_with_fr(&DFR::new())
    }

    fn convert_with_fr<DFR: Framerate>(&self, fr: &DFR) -> Timecode<DFR> {
        let count = self.to_frame_count() as u64;

        //new frame count = old frame count * new_framerate / old_framerate
        //new = old * (new_fr_num / new_fr_denom) / (old_fr_num / old_fr_denom)
        //new = old * (new_fr_num / new_fr_denom) * (old_fr_denom / old_fr_num)

        let new_fr = count * fr.fr_num() * self.framerate.fr_denom();
        let new_fr = new_fr / fr.fr_denom() / self.framerate.fr_num();

        Timecode::from_frames(&Frames(new_fr.try_into().expect("Too large")), fr)
    }

    fn convert_with_start<DFR>(&self, start: Self) -> Timecode<DFR>
    where
        DFR: Framerate + ConstFramerate,
    {
        self.convert_with_start_fr(start, &DFR::new())
    }

    fn convert_with_start_fr<DFR>(&self, start: Self, fr: &DFR) -> Timecode<DFR>
    where
        DFR: Framerate,
    {
        let self_count = self.to_frame_count();
        let start_count = start.to_frame_count();

        if self_count < start_count {
            panic!("input timecode is less than start");
        }

        let new_tc: Timecode<FR> =
            Timecode::from_frames(&Frames(self_count - start_count), &self.framerate);
        let new_tc: Timecode<DFR> = new_tc.convert_with_fr(fr);

        let new_start: Timecode<DFR> = start.convert_with_fr(fr);

        new_tc + new_start
    }
}

//simple function to give division with remainder.
fn div_rem(a: FrameCount, b: FrameCount) -> (FrameCount, FrameCount) {
    (a / b, a % b)
}

impl<FR: Framerate> ToFrames<FR> for Timecode<FR> {
    //This should be inlined after monomorphization so we shouldn't need inline
    fn to_frame_count(&self) -> FrameCount {
        let max_frame = self.framerate.max_frame() as FrameCount;
        let mut frame_count: FrameCount = 0;
        frame_count += self.h as FrameCount * 60 * 60 * max_frame;
        frame_count += self.m as FrameCount * 60 * max_frame;
        frame_count += self.s as FrameCount * max_frame;
        frame_count += self.f as FrameCount;

        if self.framerate.is_dropframe() {
            let minute_count = self.h as FrameCount * 60 + self.m as FrameCount;
            let frames_lost_per_skip = 2;
            //every 10 minutes, we /dont/ skip a frame. so count the number of times
            //that happens. This should always be <= minute_count or we will panic.
            let dropskip_count = minute_count / 10;
            frame_count -= (minute_count - dropskip_count) * frames_lost_per_skip;
        }

        frame_count
    }

    fn from_frames(&Frames(mut frame_count): &Frames, fr: &FR) -> Self {
        let max_frame = fr.max_frame() as FrameCount;
        if fr.fr_num() == 30000 && fr.fr_denom() == 1001 {
            //17982 = 29.97 * 60 * 10
            let (d, mut m) = div_rem(frame_count, 17982);
            if m < 2 {
                m += 2;
            }
            frame_count += 18 * d + 2 * ((m - 2) / 1798)
        } else if fr.is_dropframe() {
            panic!("Dropframe logic for non-29.97 not implemented");
        }

        let f = (frame_count % max_frame) as FrameCount;
        frame_count /= max_frame;
        let s = (frame_count % 60) as u8;
        frame_count /= 60;
        let m = (frame_count % 60) as u8;
        frame_count /= 60;
        let h = frame_count as u8;

        Timecode {
            f,
            s,
            m,
            h,
            framerate: *fr,
        }
    }
}

impl ToFrames<()> for Frames {
    fn to_frame_count(&self) -> FrameCount {
        self.0
    }

    fn from_frames(f: &Frames, _: &()) -> Self {
        *f
    }
}

impl<FR: Framerate> std::ops::Add<Timecode<FR>> for Timecode<FR> {
    type Output = Self;

    fn add(self, rhs: Timecode<FR>) -> Self::Output {
        self.try_add(rhs).expect("Failed to add")
    }
}

///Error returned when adding two incompatable [`DynFramerate`] timecodes
#[derive(Debug)]
pub struct FramerateMismatch;

impl<FR: Framerate> Timecode<FR> {
    fn try_add(self, rhs: Timecode<FR>) -> Result<Self, FramerateMismatch> {
        if self.framerate.fr_num() != rhs.framerate.fr_num()
            || self.framerate.fr_denom() != rhs.framerate.fr_denom()
        {
            return Err(FramerateMismatch);
        }
        let frames = Frames(self.to_frame_count()) + Frames(rhs.to_frame_count());
        Ok(Timecode::from_frames(&frames, &self.framerate))
    }
}

impl<FR: Framerate> std::ops::Add<Frames> for Timecode<FR> {
    type Output = Self;

    fn add(self, rhs: Frames) -> Self::Output {
        let frames = Frames(self.to_frame_count()) + rhs;
        Timecode::from_frames(&frames, &self.framerate)
    }
}

impl std::ops::Add<Frames> for Frames {
    type Output = Frames;

    fn add(self, rhs: Frames) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<FR: Framerate> std::ops::Sub<Frames> for Timecode<FR> {
    type Output = Self;

    ///PANIC: if rhs > lhs
    fn sub(self, rhs: Frames) -> Self::Output {
        let frame_count = self.to_frame_count();
        let frames = Frames(frame_count) - rhs;

        Timecode::from_frames(&frames, &self.framerate)
    }
}

impl std::ops::Sub<Frames> for Frames {
    type Output = Frames;

    ///PANIC: if rhs > lhs
    fn sub(self, rhs: Frames) -> Self::Output {
        assert!(self.0 >= rhs.0);
        Self(self.0 - rhs.0)
    }
}

#[cfg(test)]
mod add_test {
    use super::*;

    #[test]
    fn add_compiles() {
        let t1: Timecode<NDF<30>> = "01:10:00:12".parse().unwrap();
        let t2: Timecode<NDF<30>> = "00:00:00:01".parse().unwrap();

        let _ = t1 + t2;
    }

    #[test]
    fn add_frames_compiles() {
        let t1: Timecode<NDF<30>> = "01:10:00:12".parse().unwrap();

        let t1 = t1 + Frames(10);
        let _ = t1 + Frames(10);
    }

    #[test]
    fn add_frames_frames_compiles() {
        let _ = Frames(20) + Frames(10);
    }

    #[test]
    fn to_frames() {
        let t1: Timecode<NDF<30>> = "00:00:01:12".parse().unwrap();

        let f = t1.to_frame_count();

        assert_eq!(f, 12 + 30);
    }

    #[test]
    fn add_tcs() {
        let t1: Timecode<NDF<30>> = "01:10:00:12".parse().unwrap();
        let t2: Timecode<NDF<30>> = "01:01:01:01".parse().unwrap();
        let t3: Timecode<NDF<30>> = "02:11:01:13".parse().unwrap();

        assert_eq!(t1 + t2, t3);
    }

    #[test]
    fn dyns() {
        let t1: Timecode<DynFramerate> = "01:10:00:12@30".parse().unwrap();
        let t2: Timecode<DynFramerate> = "00:00:00:01@30".parse().unwrap();

        let _ = t1 + t2;
    }

    #[test]
    fn dyns_mismatch() {
        let t1: Timecode<DynFramerate> = "01:10:00:12@30".parse().unwrap();
        let t2: Timecode<DynFramerate> = "00:00:00:01@25".parse().unwrap();

        dbg!(t1);
        dbg!(t2);

        assert!(t1.try_add(t2).is_err());
    }

    #[test]
    fn size_of_dyn_larger() {
        let t1: Timecode<DynFramerate> = "01:10:00:12@30".parse().unwrap();
        let t2: Timecode<NDF<30>> = "01:10:00:12".parse().unwrap();

        let a = std::mem::size_of_val(&t1);
        let b = std::mem::size_of_val(&t2);

        dbg!(a, b, t1, t2);

        assert!(a > b);
    }

    #[test]
    fn dyn_downcast() {
        let t1: Timecode<DynFramerate> = "01:10:00:12@30".parse().unwrap();
        let t2: NDF<30> = t1.framerate().try_into().unwrap();
    }

    #[test]
    fn dyn_impl_fr() {
        let t1: DynFramerate = "30".parse().unwrap();

        assert_eq!(t1.max_frame(), 30);
    }
}
