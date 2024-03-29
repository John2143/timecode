use crate::{
    parser::{Seperator, UnvalidatedTC},
    ConstFramerate, FrameCount, Framerate, Timecode,
};

type FramerateValidationResult = Result<(), TimecodeValidationError>;

///The warnings container is used to store possible unintended errors when creating the desired
///timecode. Things like incorrect dropframe separator and rounding errors when using SMPTE2308
///timecodes will be added to the provided `&self` structure.
pub trait WarningContainer {
    fn add_warning(&mut self, w: TimecodeValidationWarning);
}

impl WarningContainer for Vec<TimecodeValidationWarning> {
    fn add_warning(&mut self, w: TimecodeValidationWarning) {
        self.push(w);
    }
}

impl WarningContainer for () {
    fn add_warning(&mut self, _: TimecodeValidationWarning) {}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TimecodeValidationError {
    ///The minutes field is invalid
    InvalidMin(u8),
    ///The seconds field is invalid
    InvalidSec(u8),
    ///The frames field is invalid (can happen because target is drop-frame)
    InvalidFrames(FrameCount),
    ///This is the error received when nom fails to parse the timecode.
    ///This will never occur when you call `.validate`, as by the time you have an unvalidated
    ///timecode to call `.validate` on, it has already passed the parsing step.
    Unparsed,
    //Framerate is bad
    InvalidFramerate(Option<f64>),
}

impl std::fmt::Display for TimecodeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimecodeValidationError::InvalidMin(n) => write!(f, "Invalid minutes {}", n),
            TimecodeValidationError::InvalidSec(n) => write!(f, "Invalid seconds {}", n),
            TimecodeValidationError::InvalidFrames(n) => write!(f, "Invalid frames {}", n),
            TimecodeValidationError::Unparsed => write!(f, "Timecode cannot be parsed"),
            TimecodeValidationError::InvalidFramerate(Some(n)) => {
                write!(f, "Invalid Framerate {n}")
            }
            TimecodeValidationError::InvalidFramerate(None) => {
                write!(f, "Invalid Framerate")
            }
        }
    }
}

impl std::error::Error for TimecodeValidationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TimecodeValidationWarning {
    ///Expected a ':' where a ';' was found or vice-versa.
    MismatchSep,
}

///Used internally when calling [`UnvalidatedTC::validate`]. If `Ok(())` is returned, the
///unvalidated timecode will be directly copied into a new [`Timecode`]
pub trait ValidateableFramerate: Framerate + Copy {
    fn validate<T: WarningContainer>(
        &self,
        input_tc: &UnvalidatedTC,
        warnings: &mut T,
    ) -> Result<(), TimecodeValidationError>;
}

impl UnvalidatedTC {
    ///Take an invalidated timecode and check that it is valid when interpreted as the framerate `FR`
    ///
    ///```
    ///# use timecode::parser::UnvalidatedTC;
    ///use timecode::framerates::NDF;
    ///
    ///let raw_tc = timecode::unvalidated("01:02:00:25").expect("could not parse string into framerate");
    ///
    ///let tc = raw_tc.validate::<NDF<30>>().unwrap();
    ///
    ///assert_eq!(tc.to_string(), "01:02:00:25");
    ///assert_eq!(tc.h(), 1);
    ///assert_eq!(tc.m(), 2);
    ///assert_eq!(tc.s(), 0);
    ///assert_eq!(tc.f(), 25);
    ///```
    pub fn validate<FR: ValidateableFramerate + ConstFramerate>(
        &self,
    ) -> Result<Timecode<FR>, TimecodeValidationError> {
        self.validate_with_fr(&FR::new())
    }

    ///Same as validate, but with a dynamic framerate parameter
    pub fn validate_with_fr<FR: ValidateableFramerate>(
        &self,
        fr: &FR,
    ) -> Result<Timecode<FR>, TimecodeValidationError> {
        fr.validate(&self, &mut ()).map(|_| {
            let UnvalidatedTC { h, m, s, f, .. } = *self;

            Timecode {
                h,
                m,
                s,
                f,
                framerate: *fr,
            }
        })
    }

    ///This validates the timecode while returning warnings about potentially incorrect timecodes.
    ///
    ///In this example, `01:02:00:25` is valid for both formats, but the seperator should be `;`
    ///when the framerate is drop frame.
    ///
    ///NOTE: this allocates only if there is a timecode warning, otherwise it is as cheap as
    ///validate
    ///```
    ///# use timecode::{framerates::*, };
    ///let raw_tc = timecode::unvalidated("01:02:00:25").unwrap();
    ///
    ///let (tc, warnings) = raw_tc.validate_with_warnings::<NDF<30>>().unwrap();
    ///assert!(warnings.is_empty());
    ///
    ///let (tc, warnings) = raw_tc.validate_with_warnings::<DF2997>().unwrap();
    ///assert_eq!(tc.to_string(), "01:02:00;25");
    ///assert!(warnings.contains(&timecode::validate::TimecodeValidationWarning::MismatchSep));
    ///```
    pub fn validate_with_warnings<FR: ValidateableFramerate + ConstFramerate>(
        &self,
    ) -> Result<(Timecode<FR>, Vec<TimecodeValidationWarning>), TimecodeValidationError> {
        self.validate_with_warnings_fr(&FR::new())
    }

    ///Same as validate_with_warnings, but with a dynamic framerate parameter
    ///```
    ///# use timecode::{framerates::*, DynFramerate};
    ///let raw_tc = timecode::unvalidated("01:02:00:12").unwrap();
    ///let framerate: DynFramerate = "25".parse().unwrap();
    ///
    ///let (tc, warnings) = raw_tc.validate_with_warnings_fr(&framerate).unwrap();
    ///assert!(warnings.is_empty());
    ///
    ///let framerate: DynFramerate = "29.97".parse().unwrap();
    ///
    ///let (tc, warnings) = raw_tc.validate_with_warnings_fr(&framerate).unwrap();
    ///assert_eq!(tc.to_string(), "01:02:00;12");
    ///assert!(warnings.contains(&timecode::validate::TimecodeValidationWarning::MismatchSep));
    ///```
    pub fn validate_with_warnings_fr<FR: ValidateableFramerate>(
        &self,
        fr: &FR,
    ) -> Result<(Timecode<FR>, Vec<TimecodeValidationWarning>), TimecodeValidationError> {
        let mut warnings = vec![];
        fr.validate(&self, &mut warnings).map(|_| {
            let UnvalidatedTC { h, m, s, f, .. } = *self;

            (
                Timecode {
                    h,
                    m,
                    s,
                    f,
                    framerate: *fr,
                },
                warnings,
            )
        })
    }

    ///Directly turn an unvalidated timecode into a validated timecode object
    ///
    ///# Safety
    ///
    ///The unvalidated timecode must hold all the SMPTE invariants. The timecode seperator does not
    ///have to match.
    ///
    ///```
    ///# use timecode::framerates::NDF;
    ///# use timecode::parser;
    ///# use std::convert::TryInto;
    ///let raw_tc = parser::UnvalidatedTC {
    ///    h: 1, m: 2, s: 0, f: 25,
    ///    seperator: ';'.try_into().unwrap()
    ///};
    ///
    ///let tc = unsafe { raw_tc.validate_unchecked::<NDF<30>>() };
    ///
    ///assert_eq!(tc.to_string(), "01:02:00:25");
    ///```
    pub unsafe fn validate_unchecked<FR: Framerate + ConstFramerate>(&self) -> Timecode<FR> {
        self.validate_unchecked_with_fr(&FR::new())
    }

    ///see validate_unchecked
    pub unsafe fn validate_unchecked_with_fr<FR: Framerate>(&self, fr: &FR) -> Timecode<FR> {
        let UnvalidatedTC { h, m, s, f, .. } = *self;

        Timecode {
            h,
            m,
            s,
            f,
            framerate: *fr,
        }
    }
}

fn helper_v_ms(m: u8, s: u8) -> Result<(), TimecodeValidationError> {
    if m >= 60 {
        return Err(TimecodeValidationError::InvalidMin(m));
    }

    if s >= 60 {
        return Err(TimecodeValidationError::InvalidSec(s));
    }

    Ok(())
}

fn helper_v_sep<FR: Framerate>(
    seperator: Seperator,
    fr: &FR,
) -> Result<(), TimecodeValidationWarning> {
    if fr.to_sep() != seperator.into() {
        return Err(TimecodeValidationWarning::MismatchSep);
    }

    Ok(())
}

fn helper_v_max_frame<FR: Framerate>(
    f: FrameCount,
    fr: &FR,
) -> Result<(), TimecodeValidationError> {
    if fr.max_frame() <= f {
        Err(TimecodeValidationError::InvalidFrames(f))
    } else {
        Ok(())
    }
}

///drop frame rules are the same regardless of framerate.
fn helper_v_drop_frame(
    _drop_frames: FrameCount,
    m: u8,
    s: u8,
    f: FrameCount,
) -> Result<(), TimecodeValidationError> {
    //TODO should this be drop_frames?
    if m % 10 != 0 && s == 0 && f < 2 {
        return Err(TimecodeValidationError::InvalidFrames(f));
    }

    Ok(())
}

impl<F: Framerate + Copy> ValidateableFramerate for F {
    fn validate<T: WarningContainer>(
        &self,
        input_tc: &UnvalidatedTC,
        warnings: &mut T,
    ) -> FramerateValidationResult {
        let UnvalidatedTC {
            m, s, f, seperator, ..
        } = *input_tc;

        helper_v_ms(m, s)?;
        helper_v_sep(seperator, self)
            .err()
            .map(|e| warnings.add_warning(e));

        if let Some(drop_frames) = self.drop_frames() {
            helper_v_drop_frame(drop_frames, m, s, f)?;
        }

        helper_v_max_frame(f, self)?;

        Ok(())
    }
}
