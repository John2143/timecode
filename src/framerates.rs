use crate::{ConstFramerate, FrameCount};
use std::convert::TryFrom;

pub type DF2997 = DF<30>;
pub type DF5994 = DF<60>;
pub type NDF30 = NDF<30>;
pub type NDF25 = NDF<25>;
pub type NDF50 = NDF<25>;
pub type NDF2398 = NDF<24>;

fn is_valid_df_count(frames: FrameCount) -> bool {
    frames.rem_euclid(30) == 0
}

///Dropframe timecode, with framerate stored at runtime. Must be multiple of 30.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DFDyn(FrameCount);
///Non-drop timecode, with framerate stored at runtime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NDFDyn(FrameCount);

impl DFDyn {
    fn new(fc: FrameCount) -> Option<Self> {
        if is_valid_df_count(fc) {
            Some(Self(fc))
        } else {
            None
        }
    }
}

impl NDFDyn {
    fn new(fc: FrameCount) -> Self {
        Self(fc)
    }
}

impl crate::Framerate for DFDyn {
    fn to_sep(&self) -> char {
        ';'
    }

    fn max_frame(&self) -> FrameCount {
        self.0
    }

    fn drop_frames(&self) -> Option<FrameCount> {
        Some(self.0 / 15) //30 = 2, 60 = 4, etc
    }

    fn fr_ratio(&self) -> f32 {
        self.fr_num() as f32 / self.fr_denom() as f32
    }

    fn fr_num(&self) -> u64 {
        (self.0 as u64) * 1000
    }

    fn fr_denom(&self) -> u64 {
        1001
    }
}

impl crate::Framerate for NDFDyn {
    fn to_sep(&self) -> char {
        ':'
    }

    fn max_frame(&self) -> FrameCount {
        self.0
    }

    fn drop_frames(&self) -> Option<FrameCount> {
        None
    }

    fn fr_ratio(&self) -> f32 {
        self.fr_num() as f32 / self.fr_denom() as f32
    }

    fn fr_num(&self) -> u64 {
        self.0 as u64
    }

    fn fr_denom(&self) -> u64 {
        1
    }
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

impl<const FRAMES: FrameCount> TryFrom<DFDyn> for DF<FRAMES> {
    type Error = ();

    fn try_from(value: DFDyn) -> Result<Self, Self::Error> {
        if value.0 == FRAMES {
            Ok(Self)
        } else {
            Err(())
        }
    }
}

impl<const FRAMES: FrameCount> TryFrom<NDFDyn> for NDF<FRAMES> {
    type Error = ();

    fn try_from(value: NDFDyn) -> Result<Self, Self::Error> {
        if value.0 == FRAMES {
            Ok(Self)
        } else {
            Err(())
        }
    }
}

impl<const FRAMES: FrameCount> crate::Framerate for NDF<FRAMES> {
    fn to_sep(&self) -> char {
        self.as_dyn().to_sep()
    }

    fn max_frame(&self) -> FrameCount {
        self.as_dyn().max_frame()
    }

    fn drop_frames(&self) -> Option<FrameCount> {
        self.as_dyn().drop_frames()
    }

    fn fr_ratio(&self) -> f32 {
        self.as_dyn().fr_ratio()
    }

    fn fr_num(&self) -> u64 {
        self.as_dyn().fr_num()
    }

    fn fr_denom(&self) -> u64 {
        self.as_dyn().fr_denom()
    }
}

impl<const FRAMES: FrameCount> crate::Framerate for DF<FRAMES> {
    fn to_sep(&self) -> char {
        self.as_dyn().to_sep()
    }

    fn max_frame(&self) -> FrameCount {
        self.as_dyn().max_frame()
    }

    fn drop_frames(&self) -> Option<FrameCount> {
        self.as_dyn().drop_frames()
    }

    fn fr_ratio(&self) -> f32 {
        self.as_dyn().fr_ratio()
    }

    fn fr_num(&self) -> u64 {
        self.as_dyn().fr_num()
    }

    fn fr_denom(&self) -> u64 {
        self.as_dyn().fr_denom()
    }
}

impl<const FRAMES: FrameCount> NDF<FRAMES> {
    fn as_dyn(&self) -> NDFDyn {
        NDFDyn(FRAMES)
    }
}

impl<const FRAMES: FrameCount> DF<FRAMES> {
    fn as_dyn(&self) -> DFDyn {
        DFDyn(FRAMES)
    }
}

impl TryFrom<&DynFramerate> for NDFDyn {
    type Error = ();

    fn try_from(value: &DynFramerate) -> Result<Self, Self::Error> {
        if let &DynFramerate::NDF(n) = value {
            return Ok(NDFDyn(n));
        }

        Err(())
    }
}

impl TryFrom<&DynFramerate> for DFDyn {
    type Error = ();

    fn try_from(value: &DynFramerate) -> Result<Self, Self::Error> {
        if let &DynFramerate::DF(n) = value {
            return Ok(DFDyn(n));
        }

        Err(())
    }
}

impl<const FRAMES: FrameCount> TryFrom<&DynFramerate> for NDF<FRAMES> {
    type Error = ();

    fn try_from(value: &DynFramerate) -> Result<Self, Self::Error> {
        if let &DynFramerate::NDF(n) = value {
            return Self::try_from(NDFDyn(n));
        }

        Err(())
    }
}

impl<const FRAMES: FrameCount> TryFrom<&DynFramerate> for DF<FRAMES> {
    type Error = ();

    fn try_from(value: &DynFramerate) -> Result<Self, Self::Error> {
        if let &DynFramerate::DF(n) = value {
            return Self::try_from(DFDyn(n));
        }

        Err(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynFramerate {
    DF(FrameCount),
    NDF(FrameCount),
}

impl crate::Framerate for DynFramerate {
    fn to_sep(&self) -> char {
        match self {
            DynFramerate::DF(n) => DFDyn(*n).to_sep(),
            DynFramerate::NDF(n) => NDFDyn(*n).to_sep(),
        }
    }

    fn max_frame(&self) -> FrameCount {
        match self {
            DynFramerate::DF(n) => DFDyn(*n).max_frame(),
            DynFramerate::NDF(n) => NDFDyn(*n).max_frame(),
        }
    }

    fn drop_frames(&self) -> Option<FrameCount> {
        match self {
            DynFramerate::DF(n) => DFDyn(*n).drop_frames(),
            DynFramerate::NDF(n) => NDFDyn(*n).drop_frames(),
        }
    }

    fn fr_ratio(&self) -> f32 {
        match self {
            DynFramerate::DF(n) => DFDyn(*n).fr_ratio(),
            DynFramerate::NDF(n) => NDFDyn(*n).fr_ratio(),
        }
    }

    fn fr_num(&self) -> u64 {
        match self {
            DynFramerate::DF(n) => DFDyn(*n).fr_num(),
            DynFramerate::NDF(n) => NDFDyn(*n).fr_num(),
        }
    }

    fn fr_denom(&self) -> u64 {
        match self {
            DynFramerate::DF(n) => DFDyn(*n).fr_denom(),
            DynFramerate::NDF(n) => NDFDyn(*n).fr_denom(),
        }
    }
}

impl std::str::FromStr for DynFramerate {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use DynFramerate::*;

        //if it can be parsed as an integer, assume it is NDF
        if let Ok(fr) = s.parse() {
            return Ok(NDF(fr));
        }

        let special_framerates = [(29.97, DF(30)), (23.98, NDF(24)), (59.97, DF(60))];
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

            //if we are close to a multiple of 29.97, use dropframe
            let k = float / 29.97;
            if (k - k.round()).abs() < EPISILON {
                return Ok(DF((k.round() as FrameCount) * 30));
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
        assert_eq!(s, DynFramerate::DF(30));
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

    #[test]
    fn read_fr_high_df() {
        let s: DynFramerate = "239.76".parse().unwrap();
        assert_eq!(s, DynFramerate::DF(240));
    }
}

#[cfg(test)]
mod construct_framerates {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn make_simple() {
        let _ = DFDyn::new(30).unwrap();
        let _ = DFDyn::new(60).unwrap();
        let _ = NDFDyn::new(30);
        let _ = NDFDyn::new(60);
        let _ = NDFDyn::new(25);
        let _ = NDFDyn::new(50);
    }

    #[test]
    #[should_panic]
    fn make_bad_df() {
        let _ = DFDyn::new(23).unwrap();
    }

    #[test]
    fn upcast() {
        let n = NDFDyn(30);
        if let Ok(NDF::<30>) = n.try_into() {
        } else {
            panic!("30");
        }
        if let Ok(NDF::<33>) = n.try_into() {
            panic!("33");
        }
    }

    #[test]
    fn read_fr_high_df() {
        let s: DynFramerate = "239.76".parse().unwrap();
        assert_eq!(s, DynFramerate::DF(240));
    }
}
