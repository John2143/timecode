#![allow(dead_code)]
//!
//!
//!# Quickstart
//!
//!```
//!use timecode::{framerates::*, Timecode};
//!
//!//The fastest way to get started is to parse a timecode directly with FromStr.
//!let tc: Timecode<NDF30> = "01:02:00:25".parse().unwrap();
//!
//!assert_eq!(tc.to_string(), "01:02:00:25");
//!assert_eq!(tc.h(), 1);
//!assert_eq!(tc.m(), 2);
//!assert_eq!(tc.s(), 0);
//!assert_eq!(tc.f(), 25);
//!```
//!
//!```
//!use timecode::{framerates::*, };
//!//Parse a string into an Option<UnvalidatedTimecode>
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
//!//You can also use the validate_with_warnings function to get warnings, if there are any
//!let (tc, warnings) = raw_tc.validate_with_warnings::<NDF30>().unwrap();
//!assert!(warnings.is_empty());
//!
//!//01:02:00:25 is not a valid 2398 timecode.
//!let invalid_tc = raw_tc.validate::<NDF2398>();
//!assert!(invalid_tc.is_err());
//!
//!//01:02:00:25 is a valid 29.97 timecode, but it doesn't use the standard ';' seperator.
//!let (tc, warnings) = raw_tc.validate_with_warnings::<DF2997>().unwrap();
//!
//!assert_eq!(tc.to_string(), "01:02:00;25");
//!assert!(warnings.contains(&timecode::TimecodeValidationWarning::MismatchSep));
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
    ($i: ident = $rep: expr, $sep: expr) => {
        pub struct $i;

        impl crate::Framerate for $i {
            #[inline]
            fn to_str() -> &'static str {
                $rep
            }

            fn to_sep() -> char {
                $sep
            }
        }
    };
}

pub trait Framerate {
    fn to_str() -> &'static str;
    fn to_sep() -> char;
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

pub trait FramerateValidation: Framerate {
    fn validate<T: validate::WarningContainer>(
        input_tc: &UnvalidatedTC,
        warns: &mut T,
    ) -> Result<(), TimecodeValidationError>;
}

pub mod framerates {
    framerate_impl! {NDF30 = "30", ':'}
    framerate_impl! {NDF2398 = "23.98", ':'}
    framerate_impl! {DF2997 = "29.97", ';'}
}

pub struct Timecode<FR> {
    h: u8,
    m: u8,
    s: u8,
    f: u8,
    framerate: PhantomData<FR>,
}

impl<T: Framerate> Display for Timecode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:02}:{:02}:{:02}{}{:02}",
            self.h,
            self.m,
            self.s,
            T::to_sep(),
            self.f
        )?;
        Ok(())
    }
}

impl<T> Timecode<T> {
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

impl<T: FramerateValidation> std::str::FromStr for Timecode<T> {
    type Err = TimecodeValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tc = unvalidated(s).ok_or(TimecodeValidationError::Unparsed)?;

        tc.validate()
    }
}
