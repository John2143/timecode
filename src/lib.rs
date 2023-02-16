#![allow(dead_code)]
//!This is a library to handle SMPTE timecodes. See [`Timecode`].
//!
//!# Quickstart
//!
//!The fastest way to get started is to parse a timecode directly with [`str::parse`](std::primitive::str::parse).
//!
//!```
//!use timecode::{framerates::*, Timecode, Convert};
//!
//!let tc: Timecode<NDF<50>> = "01:02:00:25".parse().expect("Couldn't convert to NDF50 timecode");
//!
//!assert_eq!(tc.h(), 1);
//!assert_eq!(tc.m(), 2);
//!assert_eq!(tc.s(), 0);
//!assert_eq!(tc.f(), 25);
//!assert_eq!(tc.to_string(), "01:02:00:25");
//!
//!let converted: Timecode<DF2997> = tc.convert();
//!assert_eq!(converted.to_string(), "01:02:00;15");
//!```
//!
//!If not all your input are known at compile time, use [`framerates::DynFramerate`]. See also
//![`Convert`].
//!```
//!use timecode::{framerates::*, Timecode, Convert};
//!
//!let tc = Timecode::new_with_fr("01:02:15:23", "30").expect("Couldn't convert to NDF30 timecode");
//!
//!assert_eq!(tc.framerate().fr_ratio(), 30.0);
//!assert!(!tc.framerate().is_dropframe());
//!
//!
//!let tc = Timecode::new_with_fr("01:02:15;23", "29.97").expect("Couldn't convert to DF30 timecode");
//!assert_eq!(tc.framerate(), &DF::<30>);
//!assert!(tc.framerate().is_dropframe());
//!
//!let framerate: DynFramerate = "25".parse().unwrap();
//!//OR
//!let framerate = DynFramerate::new_ndf(25);
//!
//!//Pass in framerate explicitly
//!let converted1 = tc.convert_with_fr(&framerate);
//!//Implicitly get framerate from destination type
//!let converted2: Timecode<NDF<25>> = tc.convert();
//!assert_eq!(converted1.to_string(), "01:02:15:19");
//!assert_eq!(converted2.to_string(), "01:02:15:19");
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
//!let invalid_tc = timecode::unvalidated("01:02:00;01").unwrap().validate::<DF<30>>();
//!assert!(invalid_tc.is_err());
//!```

use std::{convert::TryInto, fmt::Display, str::FromStr};

pub mod framerates;
#[cfg(feature = "jni")]
pub mod java;
#[cfg(feature = "javascript")]
pub mod javascript;
pub mod parser;
#[cfg(feature = "python")]
pub mod python;
pub mod validate;

pub use framerates::*;
pub use parser::unvalidated;
pub use validate::ValidateableFramerate;

use validate::TimecodeValidationError;

//24 hours * 60 * 60 * 120 still has lots of room in a u32
pub type FrameCount = u32;

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

impl<FR: validate::ValidateableFramerate + ConstFramerate> FromStr for Timecode<FR> {
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
            .map_err(|_| TimecodeValidationError::InvalidFramerate(None))?;

        tc.validate_with_fr(&d)
    }
}

impl Timecode<DynFramerate> {
    ///Construct a `Timecode<DynFramerate>` with timecode and famerate as string inputs
    pub fn new_with_fr(timecode: &str, framerate: &str) -> Result<Self, TimecodeValidationError> {
        let tc = unvalidated(timecode).ok_or(TimecodeValidationError::Unparsed)?;
        let d: DynFramerate = framerate
            .parse()
            .map_err(|_| TimecodeValidationError::InvalidFramerate(None))?;

        tc.validate_with_fr(&d)
    }
}

///Things that can be converted to a frame count
///
///Both [`Timecode`] and [`Frames`] implement this.
pub trait ToFrames<FR> {
    fn to_frame_count(&self) -> FrameCount;
    fn from_frames(f: &Frames, fr: &FR) -> Self;
}

///This trait is for converting things between different framerates. It has two types of functions:
/// `convert` and `convert_with_start` use const framerates.
///```
///# use timecode::framerates::*;
///# use timecode::{Timecode, Convert};
///
///let x: Timecode<NDF<30>> = "00:01:02:03".parse().unwrap();
///let y: Timecode<NDF<25>> = x.convert();
///assert_eq!(y.to_string(), "00:01:02:02");
///
///let start: Timecode<DF2997> = "01:00:00;00".parse().unwrap();
///let x: Timecode<DF2997> = "01:01:02;27".parse().unwrap();
///let y: Timecode<NDF2398> = x.convert_with_start(&start);
///assert_eq!(y.to_string(), "01:01:02:20");
///
///let y_wrong: Timecode<NDF2398> = x.convert();
///assert_eq!(y_wrong.to_string(), "01:01:02:21");
///```
///
///If, instead, you have dynamic timecodes, try `convert_with_fr` or `convert_with_start_fr`
///```
///# use timecode::framerates::*;
///# use timecode::{Timecode, Convert, unvalidated};
///
///let x: Timecode<DynFramerate> = "00:01:02:03@30".parse().unwrap();
///let y: Timecode<DynFramerate> = x.convert_with_fr(&DynFramerate::new_ndf(25));
///assert_eq!(y.to_string(), "00:01:02:02");
///assert_eq!(y.framerate(), &DynFramerate::new_ndf(25));
///
///let start = Timecode::new_with_fr("01:00:00;00", "29.97").unwrap();
///let x = Timecode::new_with_fr("01:01:02;27", "29.97").unwrap();
///let y1 = x.convert_with_start_fr(&start, &DynFramerate::new_ndf(24));
///assert_eq!(y1.to_string(), "01:01:02:20");
///assert_eq!(y1.framerate().fr_ratio(), 24.0);
///let y2: Timecode<NDF<24>> = x.convert_with_start(&start);
///assert_eq!(y2.to_string(), "01:01:02:20");
///assert_eq!(y2.framerate().fr_ratio(), 24.0);
///assert_eq!(y1.framerate(), y2.framerate());
///
///let y_wrong: Timecode<NDF2398> = x.convert();
///assert_eq!(y_wrong.to_string(), "01:01:02:21");
///```
pub trait Convert {
    //TODO: When HKT/GATs are merged, make this a GAT
    //type Output<DFR>;
    fn convert<DFR: Framerate + ConstFramerate>(&self) -> Timecode<DFR>;
    fn convert_with_fr<DFR: Framerate>(&self, framerate: &DFR) -> Timecode<DFR>;
    fn convert_with_start<DFR: Framerate + ConstFramerate>(&self, start: &Self) -> Timecode<DFR>;
    fn convert_with_start_fr<DFR: Framerate>(&self, start: &Self, framerate: &DFR)
        -> Timecode<DFR>;
}
impl<FR: Framerate> Convert for Timecode<FR> {
    //type Output<DFR> = Timecode<DFR>;

    fn convert<DFR: Framerate + ConstFramerate>(&self) -> Timecode<DFR> {
        self.convert_with_fr(&DFR::new())
    }

    fn convert_with_fr<DFR: Framerate>(&self, fr: &DFR) -> Timecode<DFR> {
        let count = self.to_frame_count() as u64;

        //new frame count = old frame count * new_framerate / old_framerate
        //new = old * (new_fr_num / new_fr_denom) / (old_fr_num / old_fr_denom)
        //new = old * (new_fr_num / new_fr_denom) * (old_fr_denom / old_fr_num)

        let new_fr = count * fr.fr_num() * self.framerate().fr_denom();
        let new_fr = new_fr / fr.fr_denom() / self.framerate().fr_num();

        Timecode::from_frames(&Frames(new_fr.try_into().expect("Too large")), fr)
    }

    fn convert_with_start<DFR>(&self, start: &Self) -> Timecode<DFR>
    where
        DFR: Framerate + ConstFramerate,
    {
        self.convert_with_start_fr(start, &DFR::new())
    }

    fn convert_with_start_fr<DFR>(&self, start: &Self, fr: &DFR) -> Timecode<DFR>
    where
        DFR: Framerate,
    {
        let self_count = self.to_frame_count();
        let start_count = start.to_frame_count();

        if self_count < start_count {
            panic!("input timecode is less than start");
        }

        let new_tc: Timecode<FR> =
            Timecode::from_frames(&Frames(self_count - start_count), self.framerate());
        let new_tc: Timecode<DFR> = new_tc.convert_with_fr(fr);

        let new_start: Timecode<DFR> = start.convert_with_fr(fr);

        new_tc + new_start
    }
}

/*
 * https://github.com/FFmpeg/FFmpeg/blob/master/libavutil/timecode.c
 * int av_timecode_adjust_ntsc_framenum2(int framenum, int fps)
 * {
 *     /* only works for multiples of NTSC 29.97 */
 *     int drop_frames = 0;
 *     int d, m, frames_per_10mins;
 *
 *     if (fps && fps % 30 == 0) {
 *         drop_frames = fps / 30 * 2;
 *         frames_per_10mins = fps / 30 * 17982;
 *     } else
 *         return framenum;
 *
 *     d = framenum / frames_per_10mins;
 *     m = framenum % frames_per_10mins;
 *
 *     return framenum + 9U * drop_frames * d + drop_frames * ((m - drop_frames) / (frames_per_10mins / 10));
 * }
 */

//simple function to give division with remainder.
fn div_rem(a: FrameCount, b: FrameCount) -> (FrameCount, FrameCount) {
    (a / b, a % b)
}

fn adjust_frame_count(drop_frames: u32, frame_count: u32) -> u32 {
    let frames_per_10_mins = drop_frames * (17982 / 2);
    let (d, mut m) = div_rem(frame_count, frames_per_10_mins);

    if m < drop_frames {
        m += drop_frames;
    }

    frame_count
        + 9 * drop_frames * d
        + drop_frames * ((m - drop_frames) / (frames_per_10_mins / 10))
}

impl<FR: Framerate> ToFrames<FR> for Timecode<FR> {
    //This should be inlined after monomorphization so we shouldn't need inline
    fn to_frame_count(&self) -> FrameCount {
        let max_frame = self.framerate().max_frame() as FrameCount;
        let mut frame_count: FrameCount = 0;
        frame_count += self.h as FrameCount * 60 * 60 * max_frame;
        frame_count += self.m as FrameCount * 60 * max_frame;
        frame_count += self.s as FrameCount * max_frame;
        frame_count += self.f as FrameCount;

        if let Some(drop_frames) = self.framerate().drop_frames() {
            let minute_count = self.h as FrameCount * 60 + self.m as FrameCount;
            //every 10 minutes, we /dont/ skip a frame. so count the number of times
            //that happens. This should always be <= minute_count or we will panic.
            let dropskip_count = minute_count / 10;
            frame_count -= (minute_count - dropskip_count) * drop_frames;
        }

        frame_count
    }

    fn from_frames(&Frames(mut frame_count): &Frames, fr: &FR) -> Self {
        let max_frame = fr.max_frame() as FrameCount;

        if let Some(drop_frames) = fr.drop_frames() {
            frame_count = adjust_frame_count(drop_frames, frame_count);
        };

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
        if self.framerate() != rhs.framerate() {
            return Err(FramerateMismatch);
        }
        let frames = Frames(self.to_frame_count()) + Frames(rhs.to_frame_count());
        Ok(Timecode::from_frames(&frames, self.framerate()))
    }
}

impl<FR: Framerate> std::ops::Add<Frames> for Timecode<FR> {
    type Output = Self;

    fn add(self, rhs: Frames) -> Self::Output {
        let frames = Frames(self.to_frame_count()) + rhs;
        Timecode::from_frames(&frames, self.framerate())
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

        Timecode::from_frames(&frames, self.framerate())
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

impl<FR1> PartialEq<Timecode<FR1>> for Timecode<DynFramerate>
where
    FR1: Framerate + ConstFramerate,
{
    fn eq(&self, other: &Timecode<FR1>) -> bool {
        self.h() == other.h()
            && self.m() == other.m()
            && self.s() == other.s()
            && self.f() == other.f()
            && self.framerate() == other.framerate()
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
        let tf = *t1.framerate();
        let _: NDF<30> = tf.try_into().unwrap();
        let _ = TryInto::<NDF<25>>::try_into(tf).unwrap_err();
    }

    #[test]
    fn dyn_impl_fr() {
        let t1: DynFramerate = "30".parse().unwrap();

        assert_eq!(t1.max_frame(), 30);
    }

    #[test]
    #[ignore]
    fn compare_timecodes() {
        //let t1: Timecode<DynFramerate> = "01:10:00:12@30".parse().unwrap();
        //let t2: Timecode<NDF30> = "01:10:00:12".parse().unwrap();

        //assert_eq!(t2, t1);
    }
}
