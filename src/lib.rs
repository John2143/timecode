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

use std::{fmt::Display, marker::PhantomData};

use parser::UnvalidatedTC;

pub mod parser;
pub mod validate;

pub use parser::unvalidated;

macro_rules! framerate_impl {
    ($i: ident = $rep: expr, $sep: expr, $max_frame: expr) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $i;

        impl crate::Framerate for $i {
            #[inline]
            fn to_str() -> &'static str {
                $rep
            }

            fn to_sep() -> char {
                $sep
            }

            fn max_frame() -> u8 {
                $max_frame
            }
        }
    };
}

pub mod framerates {
    framerate_impl! {NDF30 = "30", ':', 30}
    framerate_impl! {NDF2398 = "23.98", ':', 24}
    framerate_impl! {DF2997 = "29.97", ';', 30}
}


pub trait Framerate: Copy {
    fn to_str() -> &'static str;
    fn to_sep() -> char;
    fn max_frame() -> u8;
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
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TimecodeValidationWarning {
    MismatchSep,
}

pub trait ValidateableFramerate: Framerate {
    fn validate<T: validate::WarningContainer>(
        input_tc: &UnvalidatedTC,
        warnings: &mut T,
    ) -> Result<(), TimecodeValidationError>;
}

#[derive(Copy, Debug, Eq, PartialEq, Clone)]
#[repr(transparent)]
pub struct Frames(pub usize);

#[derive(Copy, Debug, Eq, PartialEq, Clone)]
pub struct Timecode<FR> {
    h: u8,
    m: u8,
    s: u8,
    f: u8,
    framerate: PhantomData<FR>,
}

impl<FR: Framerate> Display for Timecode<FR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:02}:{:02}:{:02}{}{:02}",
            self.h,
            self.m,
            self.s,
            FR::to_sep(),
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

pub trait ToFrames {
    fn to_frames(&self) -> Frames;
}

impl<FR> ToFrames for Timecode<FR> {
    fn to_frames(&self) -> Frames {
        Frames(0)
    }
}

impl ToFrames for Frames {
    fn to_frames(&self) -> Frames {
        *self
    }
}


impl<T: ToFrames, FR> std::ops::Add<T> for Timecode<FR> {
    type Output = Timecode<FR>;

    fn add(self, rhs: T) -> Self::Output {
        self + rhs.to_frames()
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
}
