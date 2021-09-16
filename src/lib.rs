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
//!let tc: Timecode<NDF30> = "01:02:00:25".parse().expect("Couldn't convert to NDF30 timecode");
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
//!let tc = raw_tc.validate::<NDF30>().unwrap();
//!
//!assert_eq!(tc.to_string(), "01:02:00:25");
//!assert_eq!(tc.h(), 1);
//!assert_eq!(tc.m(), 2);
//!assert_eq!(tc.s(), 0);
//!assert_eq!(tc.f(), 25);
//!
//!//01:02:00:25 is not a valid 2398 timecode.
//!let invalid_tc = raw_tc.validate::<NDF2398>();
//!assert!(invalid_tc.is_err());
//!
//!//Dropframe invariants are also checked.
//!let invalid_tc = timecode::unvalidated("01:02:00;01").unwrap().validate::<DF2997>();
//!assert!(invalid_tc.is_err());
//!```

use std::{convert::TryInto, fmt::Display};

use parser::UnvalidatedTC;

pub mod parser;
pub mod validate;

pub use parser::unvalidated;

macro_rules! framerate_impl {
    ($i: ident = $rep: expr, $sep: expr, $max_frame: expr, df = $is_dropframe: expr, $fr_num: expr ; $fr_den: expr, ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $i;

        impl crate::Framerate for $i {
            fn new() -> Self {
                $i
            }

            #[inline]
            fn to_str(&self) -> &'static str {
                $rep
            }

            fn to_sep(&self) -> char {
                $sep
            }

            fn max_frame(&self) -> u8 {
                $max_frame
            }

            fn is_dropframe(&self) -> bool {
                $is_dropframe
            }

            fn framerate_ratio(&self) -> f32 {
                self.nom() as f32 / self.den() as f32
            }

            fn nom(&self) -> u64 {
                $fr_num
            }

            fn den(&self) -> u64 {
                $fr_den
            }

            fn bbox(&self) -> Box<dyn crate::Framerate> {
                Box::new($i)
            }
        }
    };
}

pub mod framerates {
    framerate_impl! {
        NDF30 = "30",
        ':', 30, df = false,
        30000 ; 1000,
    }
    framerate_impl! {
        NDF2398 = "23.98",
        ':', 24, df = false,
        24000 ; 1000,

    }
    framerate_impl! {
        DF2997 = "29.97",
        ';', 30, df = true,
        30000 ; 1001,
    }
    framerate_impl! {
        NDF25 = "25",
        ':', 25, df = false,
        25000 ; 1000,
    }
    framerate_impl! {
        NDF50 = "50",
        ':', 50, df = false,
        50000 ; 1000,
    }
}

pub trait Framerate {
    fn new() -> Self
    where
        Self: Sized;
    fn to_str(&self) -> &'static str;
    fn to_sep(&self) -> char;
    fn max_frame(&self) -> u8;
    fn is_dropframe(&self) -> bool;
    fn framerate_ratio(&self) -> f32;
    fn nom(&self) -> u64;
    fn den(&self) -> u64;
    fn bbox(&self) -> Box<dyn Framerate>;
}

pub struct FramerateMismatch;

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
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TimecodeValidationWarning {
    MismatchSep,
}

///Used internally when calling [`UnvalidatedTC::validate`]. If `Ok(())` is returned, the
///unvalidated timecode will be directly copied into a new [`Timecode`]
pub trait ValidateableFramerate: Framerate {
    fn validate<T: validate::WarningContainer>(
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
    f: u8,
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
    pub fn f(&self) -> u8 {
        self.f
    }
}

impl<FR: ValidateableFramerate> std::str::FromStr for Timecode<FR> {
    type Err = TimecodeValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tc = unvalidated(s).ok_or(TimecodeValidationError::Unparsed)?;

        tc.validate()
    }
}

pub trait ToFrames<FR> {
    fn to_frame_count(&self) -> FrameCount;
    fn from_frames(f: &Frames, fr: &FR) -> Self;
}

pub trait Convert {
    fn convert_to<DFR: Framerate>(&self) -> Timecode<DFR>;
    fn convert_with_start<DFR: Framerate>(&self, start: Self) -> Timecode<DFR>;
}

pub trait DynConvert {
    fn convert_to(&self, fr: &dyn Framerate) -> Self;
    fn convert_with_start(&self, fr: &dyn Framerate, start: Self) -> Self;
}

pub trait DynTimecodeTrait: ToFrames<Box<dyn Framerate>> + DynConvert {}

struct DynTimecode {
    h: u8,
    m: u8,
    s: u8,
    f: u8,
    framerate: Box<dyn Framerate>,
}

impl<FR: Framerate> Convert for Timecode<FR> {
    fn convert_to<DFR: Framerate>(&self) -> Timecode<DFR> {
        let count = self.to_frame_count() as u64;

        //new frame count = old frame count * new_framerate / old_framerate
        //new = old * (new_fr_num / new_fr_denom) / (old_fr_num / old_fr_denom)
        //new = old * (new_fr_num / new_fr_denom) * (old_fr_denom / old_fr_num)

        let new_fr = count * DFR::new().nom() * self.framerate.den();
        let new_fr = new_fr / DFR::new().den() / FR::new().nom();

        Timecode::from_frames(&Frames(new_fr.try_into().expect("Too large")), &DFR::new())
    }

    fn convert_with_start<DFR>(&self, start: Self) -> Timecode<DFR>
    where
        DFR: Framerate,
    {
        let self_count = self.to_frame_count();
        let start_count = start.to_frame_count();

        if self_count < start_count {
            panic!("input timecode is less than start");
        }

        let new_tc: Timecode<FR> =
            Timecode::from_frames(&Frames(self_count - start_count), &FR::new());
        let new_tc: Timecode<DFR> = new_tc.convert_to();

        let new_start: Timecode<DFR> = start.convert_to();

        new_tc + new_start
    }
}

impl DynConvert for DynTimecode {
    fn convert_to(&self, dfr: &dyn Framerate) -> Self {
        let count = self.to_frame_count() as u64;

        //new frame count = old frame count * new_framerate / old_framerate
        //new = old * (new_fr_num / new_fr_denom) / (old_fr_num / old_fr_denom)
        //new = old * (new_fr_num / new_fr_denom) * (old_fr_denom / old_fr_num)

        let new_fr = count * dfr.nom() * self.framerate.den();
        let new_fr = new_fr / dfr.den() / self.framerate.nom();

        DynTimecode::from_frames(&Frames(new_fr.try_into().expect("Too large")), &dfr.bbox())
    }

    fn convert_with_start(&self, fr: &dyn Framerate, start: Self) -> Self {
        let self_count = self.to_frame_count();
        let start_count = start.to_frame_count();

        if self_count < start_count {
            panic!("input timecode is less than start");
        }

        let new_tc: DynTimecode =
            DynTimecode::from_frames(&Frames(self_count - start_count), &self.framerate);
        let new_tc: DynTimecode = new_tc.convert_to(fr);

        let new_start: DynTimecode = start.convert_to(fr);

        new_tc + new_start
    }
}

impl ToFrames<Box<dyn Framerate>> for DynTimecode {
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

    fn from_frames(&Frames(mut frame_count): &Frames, fr: &Box<dyn Framerate>) -> Self {
        let max_frame = fr.max_frame() as FrameCount;
        if fr.nom() == 30000 && fr.den() == 1001 {
            //17982 = 29.97 * 60 * 10
            let (d, mut m) = div_rem(frame_count, 17982);
            if m < 2 {
                m += 2;
            }
            frame_count += 18 * d + 2 * ((m - 2) / 1798)
        } else if fr.is_dropframe() {
            panic!("Dropframe logic for non-29.97 not implemented");
        }

        let f = (frame_count % max_frame) as u8;
        frame_count /= max_frame;
        let s = (frame_count % 60) as u8;
        frame_count /= 60;
        let m = (frame_count % 60) as u8;
        frame_count /= 60;
        let h = frame_count as u8;

        DynTimecode {
            f,
            s,
            m,
            h,
            framerate: fr.bbox(),
        }
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
        if fr.nom() == 30000 && fr.den() == 1001 {
            //17982 = 29.97 * 60 * 10
            let (d, mut m) = div_rem(frame_count, 17982);
            if m < 2 {
                m += 2;
            }
            frame_count += 18 * d + 2 * ((m - 2) / 1798)
        } else if fr.is_dropframe() {
            panic!("Dropframe logic for non-29.97 not implemented");
        }

        let f = (frame_count % max_frame) as u8;
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
            framerate: FR::new(),
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
    type Output = Timecode<FR>;

    fn add(self, rhs: Timecode<FR>) -> Self::Output {
        let frames = Frames(self.to_frame_count()) + Frames(rhs.to_frame_count());
        Timecode::from_frames(&frames, &FR::new())
    }
}

impl std::ops::Add<DynTimecode> for DynTimecode {
    type Output = DynTimecode;

    fn add(self, rhs: Self) -> Self::Output {
        let frames = Frames(self.to_frame_count()) + Frames(rhs.to_frame_count());
        assert_eq!(self.framerate.to_str(), rhs.framerate.to_str());
        DynTimecode::from_frames(&frames, &self.framerate)
    }
}

impl<FR: Framerate> std::ops::Add<Frames> for Timecode<FR> {
    type Output = Timecode<FR>;

    fn add(self, rhs: Frames) -> Self::Output {
        let frames = Frames(self.to_frame_count()) + rhs;
        Timecode::from_frames(&frames, &FR::new())
    }
}

impl std::ops::Add<Frames> for Frames {
    type Output = Frames;

    fn add(self, rhs: Frames) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

#[cfg(test)]
mod add_test {
    use super::*;
    use crate::framerates::NDF30;

    #[test]
    fn add_compiles() {
        let t1: Timecode<NDF30> = "01:10:00:12".parse().unwrap();
        let t2: Timecode<NDF30> = "00:00:00:01".parse().unwrap();

        let _ = t1 + t2;
    }

    #[test]
    fn add_frames_compiles() {
        let t1: Timecode<NDF30> = "01:10:00:12".parse().unwrap();

        let _ = t1 + Frames(10);
        let _ = t1 + Frames(10);
    }

    #[test]
    fn add_frames_frames_compiles() {
        let _ = Frames(20) + Frames(10);
    }

    #[test]
    fn to_frames() {
        let t1: Timecode<NDF30> = "00:00:01:12".parse().unwrap();

        let f = t1.to_frame_count();

        assert_eq!(f, 12 + 30);
    }

    #[test]
    fn add_tcs() {
        let t1: Timecode<NDF30> = "01:10:00:12".parse().unwrap();
        let t2: Timecode<NDF30> = "01:01:01:01".parse().unwrap();
        let t3: Timecode<NDF30> = "02:11:01:13".parse().unwrap();

        assert_eq!(t1 + t2, t3);
    }
}
