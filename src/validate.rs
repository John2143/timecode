use crate::{
    framerates::*,
    parser::{Seperator, UnvalidatedTC},
    Framerate, FramerateValidation, Timecode, TimecodeValidationError, TimecodeValidationWarning,
};

type FramerateValidationResult = Result<(), TimecodeValidationError>;

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
    ///This checks that minutes and seconds are valid in
    pub fn validate<T: FramerateValidation>(&self) -> Result<Timecode<T>, TimecodeValidationError> {
        T::validate(&self, &mut ()).map(|_| {
            let UnvalidatedTC { h, m, s, f, .. } = *self;

            Timecode {
                h,
                m,
                s,
                f,
                framerate: std::marker::PhantomData,
            }
        })
    }

    pub fn validate_with_warnings<T: FramerateValidation>(
        &self,
    ) -> Result<(Timecode<T>, Vec<TimecodeValidationWarning>), TimecodeValidationError> {
        let mut warnings = vec![];
        T::validate(&self, &mut warnings).map(|_| {
            let UnvalidatedTC { h, m, s, f, .. } = *self;

            (
                Timecode {
                    h,
                    m,
                    s,
                    f,
                    framerate: std::marker::PhantomData,
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
    pub unsafe fn validate_unchecked<T: Framerate>(&self) -> Timecode<T> {
        let UnvalidatedTC { h, m, s, f, .. } = *self;

        Timecode {
            h,
            m,
            s,
            f,
            framerate: std::marker::PhantomData,
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

fn helper_v_sep<FR: Framerate>(seperator: Seperator) -> Result<(), TimecodeValidationWarning> {
    if FR::to_sep() != seperator.into() {
        return Err(TimecodeValidationWarning::MismatchSep);
    }

    Ok(())
}

///drop frame rules are the same regardless of framerate.
fn helper_v_drop_frame(m: u8, s: u8, f: u8) -> Result<(), TimecodeValidationError> {
    if m % 10 != 0 && s == 0 && f < 2 {
        return Err(TimecodeValidationError::InvalidFrames);
    }

    Ok(())
}

impl FramerateValidation for NDF30 {
    fn validate<T: WarningContainer>(
        input_tc: &UnvalidatedTC,
        warnings: &mut T,
    ) -> FramerateValidationResult {
        let UnvalidatedTC {
            m, s, f, seperator, ..
        } = *input_tc;

        helper_v_ms(m, s)?;
        helper_v_sep::<Self>(seperator)
            .err()
            .map(|e| warnings.add_warning(e));

        if f >= 30 {
            return Err(TimecodeValidationError::InvalidFrames);
        }

        Ok(())
    }
}

impl FramerateValidation for DF2997 {
    fn validate<T: WarningContainer>(
        input_tc: &UnvalidatedTC,
        warnings: &mut T,
    ) -> FramerateValidationResult {
        let UnvalidatedTC {
            m, s, f, seperator, ..
        } = *input_tc;

        helper_v_ms(m, s)?;
        helper_v_sep::<Self>(seperator)
            .err()
            .map(|e| warnings.add_warning(e));
        helper_v_drop_frame(m, s, f)?;

        if f >= 30 {
            return Err(TimecodeValidationError::InvalidFrames);
        }

        Ok(())
    }
}

impl FramerateValidation for NDF2398 {
    fn validate<T: WarningContainer>(
        input_tc: &UnvalidatedTC,
        warnings: &mut T,
    ) -> FramerateValidationResult {
        let UnvalidatedTC {
            m, s, f, seperator, ..
        } = *input_tc;

        helper_v_ms(m, s)?;
        helper_v_sep::<Self>(seperator)
            .err()
            .map(|e| warnings.add_warning(e));

        if f >= 24 {
            return Err(TimecodeValidationError::InvalidFrames);
        }

        Ok(())
    }
}
