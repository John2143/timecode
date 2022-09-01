use crate::FrameCount;
use std::convert::TryFrom;

pub trait Framerate: PartialEq + Copy {
    fn to_sep(&self) -> char;
    fn max_frame(&self) -> FrameCount;
    fn drop_frames(&self) -> Option<FrameCount>;
    fn fr_ratio(&self) -> f32;
    fn fr_num(&self) -> u64;
    fn fr_denom(&self) -> u64;
    fn is_dropframe(&self) -> bool {
        self.drop_frames().is_some()
    }
}

pub trait ConstFramerate {
    fn new() -> Self;
}

///29.97 DF (NTSC)
pub type DF2997 = DF<30>;
///59.94 DF (NTSC)
pub type DF5994 = DF<60>;
///30 NDF
pub type NDF30 = NDF<30>;
///25 NDF (PAL)
pub type NDF25 = NDF<25>;
///50 NDF (PAL)
pub type NDF50 = NDF<25>;
///23.98 NDF like 24fps
pub type NDF2398 = NDF<24>;

///dropframe timecodes must be multiples of 29.97, so check that the rounded value is divisible by
///30
const fn is_valid_df_count(frames: FrameCount) -> bool {
    frames.rem_euclid(30) == 0
}

///Dropframe timecode, with framerate stored at compile-time. Must be multiple of 30.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DF<const FRAMES: FrameCount>;
///Non-drop timecode, with framerate stored at compile-time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NDF<const FRAMES: FrameCount>;

impl<const FRAMES: FrameCount> ConstFramerate for NDF<FRAMES> {
    fn new() -> Self {
        Self
    }
}

impl<const FRAMES: FrameCount> ConstFramerate for DF<FRAMES> {
    fn new() -> Self {
        if is_valid_df_count(FRAMES) {
            Self
        } else {
            panic!(
                "Framerate for dropframe timecodes must be a multiple of 30, got {}",
                FRAMES
            )
        }
    }
}

impl<const FRAMES: FrameCount> crate::Framerate for NDF<FRAMES> {
    fn to_sep(&self) -> char {
        ':'
    }

    fn max_frame(&self) -> FrameCount {
        FRAMES
    }

    fn drop_frames(&self) -> Option<FrameCount> {
        None
    }

    fn fr_ratio(&self) -> f32 {
        self.fr_num() as f32 / self.fr_denom() as f32
    }

    fn fr_num(&self) -> u64 {
        FRAMES as u64
    }

    fn fr_denom(&self) -> u64 {
        1
    }
}

impl<const FRAMES: FrameCount> crate::Framerate for DF<FRAMES> {
    fn to_sep(&self) -> char {
        ';'
    }

    fn max_frame(&self) -> FrameCount {
        FRAMES
    }

    fn drop_frames(&self) -> Option<FrameCount> {
        Some(FRAMES / 15) //30 = 2, 60 = 4, etc,
    }

    fn fr_ratio(&self) -> f32 {
        self.fr_num() as f32 / self.fr_denom() as f32
    }

    fn fr_num(&self) -> u64 {
        (FRAMES as u64) * 1000
    }

    fn fr_denom(&self) -> u64 {
        1001
    }
}

impl<F> From<&F> for DynFramerate
where
    F: ConstFramerate + Framerate,
{
    fn from(s: &F) -> DynFramerate {
        DynFramerate::new(s.max_frame(), s.drop_frames().is_some()).unwrap()
    }
}

impl<F> PartialEq<F> for DynFramerate
where
    F: ConstFramerate + Framerate,
{
    fn eq(&self, other: &F) -> bool {
        DynFramerate::from(other) == *self
    }
}

impl<const FRAMES: FrameCount> TryFrom<DynFramerate> for NDF<FRAMES> {
    type Error = ();

    fn try_from(value: DynFramerate) -> Result<Self, Self::Error> {
        if !value.is_df && value.count == FRAMES {
            Ok(Self::new())
        } else {
            Err(())
        }
    }
}

impl<const FRAMES: FrameCount> TryFrom<DynFramerate> for DF<FRAMES> {
    type Error = ();

    fn try_from(value: DynFramerate) -> Result<Self, Self::Error> {
        if value.is_df && value.count == FRAMES {
            Ok(Self::new())
        } else {
            Err(())
        }
    }
}

///A framerate stored at runtime. can be either DF or NDF.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DynFramerate {
    count: FrameCount,
    is_df: bool,
}

impl DynFramerate {
    pub const fn new(count: FrameCount, is_df: bool) -> Option<Self> {
        if is_df && !is_valid_df_count(count) {
            return None;
        }

        Some(Self { count, is_df })
    }

    ///Shorthand for `Self::new(count, true)`
    #[allow(non_snake_case)]
    pub const fn try_new_df(count: FrameCount) -> Option<Self> {
        Self::new(count, true)
    }

    ///Shorthand for `Self::new(count, false)`
    #[allow(non_snake_case)]
    pub const fn new_ndf(count: FrameCount) -> Self {
        Self {
            count,
            is_df: false,
        }
    }

    ///Shorthand for `Self::new(count, true).unwrap()`
    ///PANIC: if count is not valid dropframe
    #[allow(non_snake_case)]
    pub const fn new_df(count: FrameCount) -> Self {
        if !is_valid_df_count(count) {
            panic!("Invalid dropframe framerate");
        }

        Self { count, is_df: true }
    }
}

impl crate::Framerate for DynFramerate {
    fn to_sep(&self) -> char {
        match self.is_df {
            true => ';',
            false => ':',
        }
    }

    fn max_frame(&self) -> FrameCount {
        self.count
    }

    fn drop_frames(&self) -> Option<FrameCount> {
        match self.is_df {
            true => Some(self.count / 15), //30 = 2, 60 = 4, etc,
            false => None,
        }
    }

    fn fr_ratio(&self) -> f32 {
        self.fr_num() as f32 / self.fr_denom() as f32
    }

    fn fr_num(&self) -> u64 {
        let base = self.count as u64;
        match self.is_df {
            true => base * 1000,
            false => base,
        }
    }

    fn fr_denom(&self) -> u64 {
        match self.is_df {
            true => 1001,
            false => 1,
        }
    }
}

impl std::str::FromStr for DynFramerate {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //if it can be parsed as an integer, assume it is NDF
        if let Ok(fr) = s.parse() {
            return Ok(Self::new(fr, false).unwrap());
        }

        if let Ok(float) = s.parse::<f64>() {
            const EPISILON: f64 = 0.01;

            //If it can be parsed as a float, see if it is near a whole number
            if (float - float.round()).abs() < EPISILON {
                return Ok(Self::new_ndf(float.round() as _));
            }

            const SPECIAL: &[(f64, DynFramerate)] = &[
                (23.98, DynFramerate::new_ndf(24)),
                (59.97, DynFramerate::new_df(60)),
                (29.97, DynFramerate::new_df(30)),
            ];

            //Or if it is a special framerate
            for (fr, s) in SPECIAL {
                if (float - fr).abs() < EPISILON {
                    return Ok(*s);
                }
            }

            //if we are close to a multiple of 29.97, use dropframe
            let k = float / 29.97;
            if (k - k.round()).abs() < EPISILON {
                return Ok(Self::try_new_df((k.round() as FrameCount) * 30).unwrap());
            }
        }

        Err("No known dropframe timecode")
    }
}

#[cfg(test)]
mod read_dyn_framerates {
    use crate::DynFramerate;

    #[test]
    fn read_int() {
        let s: DynFramerate = "25".parse().unwrap();
        assert_eq!(s, DynFramerate::new_ndf(25));
    }

    #[test]
    fn read_float() {
        let s: DynFramerate = "25.00".parse().unwrap();
        assert_eq!(s, DynFramerate::new_ndf(25));
    }

    #[test]
    fn read_float_df() {
        let s: DynFramerate = "29.97".parse().unwrap();
        assert_eq!(s, DynFramerate::new_df(30));
    }

    #[test]
    fn read_float_ndf_special() {
        let s: DynFramerate = "23.98".parse().unwrap();
        assert_eq!(s, DynFramerate::new_ndf(24));
    }

    #[test]
    fn read_fr_high() {
        let s: DynFramerate = "239.99".parse().unwrap();
        assert_eq!(s, DynFramerate::new_ndf(240));
    }

    #[test]
    fn read_fr_high_df() {
        let s: DynFramerate = "239.76".parse().unwrap();
        assert_eq!(s, DynFramerate::new_df(240));
    }
}

#[cfg(test)]
mod construct_framerates {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn make_simple() {
        let _ = DynFramerate::new_df(30);
        let _ = DynFramerate::new_df(60);
        let _ = DynFramerate::new_ndf(30);
        let _ = DynFramerate::new_ndf(60);
        let _ = DynFramerate::new_ndf(25);
        let _ = DynFramerate::new_ndf(50);
    }

    #[test]
    #[should_panic]
    fn make_bad_df() {
        let _ = DynFramerate::new_df(23);
    }

    #[test]
    #[should_panic]
    fn make_bad_df_unwrao() {
        let _ = DynFramerate::try_new_df(23).unwrap();
    }

    #[test]
    fn upcast() {
        let n = DynFramerate::new_ndf(30);
        if let Ok(NDF::<30>) = n.try_into() {
        } else {
            panic!("30");
        }
        if let Ok(NDF::<33>) = n.try_into() {
            panic!("33");
        }
    }

    #[test]
    fn make_dyn() {
        let s = NDF::<30>;
        let df = DynFramerate::from(&s);
        if let Ok(NDF::<30>) = df.try_into() {
        } else {
            panic!("30");
        }
    }
}
