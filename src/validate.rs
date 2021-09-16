use crate::{
    framerates::*,
    parser::{Seperator, UnvalidatedTC},
    Framerate, NewFramerate, Timecode, TimecodeValidationError, TimecodeValidationWarning,
    ValidateableFramerate,
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

impl UnvalidatedTC {
    ///Take an invalidated timecode and check that it is valid when interpreted as the framerate `FR`
    ///
    ///```
    ///# use timecode::parser::UnvalidatedTC;
    ///use timecode::framerates::NDF30;
    ///
    ///let raw_tc = timecode::unvalidated("01:02:00:25").expect("could not parse string into framerate");
    ///
    ///let tc = raw_tc.validate::<NDF30>().unwrap();
    ///
    ///assert_eq!(tc.to_string(), "01:02:00:25");
    ///assert_eq!(tc.h(), 1);
    ///assert_eq!(tc.m(), 2);
    ///assert_eq!(tc.s(), 0);
    ///assert_eq!(tc.f(), 25);
    ///```
    pub fn validate<FR: ValidateableFramerate + NewFramerate>(
        &self,
    ) -> Result<Timecode<FR>, TimecodeValidationError> {
        self.validate_dyn(&FR::new())
    }

    ///See validate_dyn
    pub fn validate_dyn<FR: ValidateableFramerate>(
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
    ///let (tc, warnings) = raw_tc.validate_with_warnings::<NDF30>().unwrap();
    ///assert!(warnings.is_empty());
    ///
    ///let (tc, warnings) = raw_tc.validate_with_warnings::<DF2997>().unwrap();
    ///assert_eq!(tc.to_string(), "01:02:00;25");
    ///assert!(warnings.contains(&timecode::TimecodeValidationWarning::MismatchSep));
    ///```
    pub fn validate_with_warnings<FR: ValidateableFramerate + NewFramerate>(
        &self,
    ) -> Result<(Timecode<FR>, Vec<TimecodeValidationWarning>), TimecodeValidationError> {
        self.validate_with_warnings_dyn(&FR::new())
    }

    ///See validate_with_warnings.
    pub fn validate_with_warnings_dyn<FR: ValidateableFramerate>(
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
    ///# use timecode::framerates::NDF30;
    ///# use timecode::parser;
    ///# use std::convert::TryInto;
    ///let raw_tc = parser::UnvalidatedTC {
    ///    h: 1, m: 2, s: 0, f: 25,
    ///    seperator: ';'.try_into().unwrap()
    ///};
    ///
    ///let tc = unsafe { raw_tc.validate_unchecked::<NDF30>() };
    ///
    ///assert_eq!(tc.to_string(), "01:02:00:25");
    ///```
    pub unsafe fn validate_unchecked<FR: Framerate + NewFramerate>(&self) -> Timecode<FR> {
        self.validate_unchecked_dyn(&FR::new())
    }

    pub unsafe fn validate_unchecked_dyn<FR: Framerate>(&self, fr: &FR) -> Timecode<FR> {
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
        return Err(TimecodeValidationError::InvalidMin);
    }

    if s >= 60 {
        return Err(TimecodeValidationError::InvalidSec);
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

fn helper_v_max_frame<FR: Framerate>(f: u8, fr: &FR) -> Result<(), TimecodeValidationError> {
    if fr.max_frame() <= f {
        Err(TimecodeValidationError::InvalidFrames)
    } else {
        Ok(())
    }
}

///drop frame rules are the same regardless of framerate.
fn helper_v_drop_frame(m: u8, s: u8, f: u8) -> Result<(), TimecodeValidationError> {
    if m % 10 != 0 && s == 0 && f < 2 {
        return Err(TimecodeValidationError::InvalidFrames);
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

        if self.is_dropframe() {
            helper_v_drop_frame(m, s, f)?;
        }

        helper_v_max_frame(f, self)?;

        Ok(())
    }
}
