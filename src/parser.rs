use std::convert::TryInto;

use nom::{
    bytes::complete::take_while_m_n,
    character::complete::{char, satisfy},
    combinator::map_res,
    error::make_error,
    sequence::{pair, tuple},
    IResult,
};

use crate::FrameCount;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Seperator {
    Semicolon,
    Colon,
}

impl Into<char> for Seperator {
    fn into(self) -> char {
        match self {
            Self::Semicolon => ';',
            Self::Colon => ':',
        }
    }
}

impl std::convert::TryFrom<char> for Seperator {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            ';' => Ok(Self::Semicolon),
            ':' => Ok(Self::Colon),
            _ => Err(()),
        }
    }
}

///This is the timecode produced directly after being parsed. It has no knowledge
///about what the target framerate is, and simply contains the data found in the string.
#[derive(Debug, PartialEq)]
pub struct UnvalidatedTC {
    pub h: u8,
    pub m: u8,
    pub s: u8,
    pub f: FrameCount,
    pub seperator: Seperator,
}

impl std::str::FromStr for UnvalidatedTC {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        match unvalidated(s) {
            Some(t) => Ok(t),
            None => Err(()),
        }
    }
}

///string to int for numbers <255
fn from_dec(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 10)
}

///takes 2-3 digits from a timecode string and parse it into int
///
///This may return an invalid value for seconds, minutes, or frames, so it is up to the user to
///validate after receiving this input.
fn tc_digits<const SIZE: usize>(input: &str) -> IResult<&str, u64> {
    map_res(take_while_m_n(2, SIZE, |c: char| c.is_digit(10)), from_dec)(input)
}

fn tc_seperator(input: &str) -> IResult<&str, Seperator> {
    //TODO get rid of the match statement somehow
    let (input, sep) = satisfy(|c| c == ';' || c == ':')(input)?;

    Ok((input, sep.try_into().unwrap()))
}

pub fn timecode_nom(input: &str) -> IResult<&str, UnvalidatedTC> {
    let parse_timecode = tuple((
        pair(tc_digits::<3>, char(':')),
        pair(tc_digits::<3>, char(':')),
        pair(tc_digits::<3>, tc_seperator),
        //up to 10 digits for frames: TODO not to spec?
        tc_digits::<10>,
    ))(input)?;

    //destructure into more readable format
    let (input, ((h, _), (m, _), (s, sep), f)) = parse_timecode;

    //Make sure we have valid values for all the parts
    let invalid = |_| nom::Err::Error(make_error(input, nom::error::ErrorKind::Alpha));
    let h = h.try_into().map_err(invalid)?;
    let m = m.try_into().map_err(invalid)?;
    let s = s.try_into().map_err(invalid)?;
    let f = f.try_into().map_err(invalid)?;

    Ok((
        input,
        UnvalidatedTC {
            h,
            m,
            s,
            f,
            seperator: sep,
        },
    ))
}

///Returns an unvalidated timecode parsed into a struct iff it matches a valid timecode format
///
/// Current valid formats:
///   HHH:MM:SS;FFF
///   HHH:MM:SS:FFF
///
/// hours and frames must be less than 256
/// hours and frames can be 2 or 3 characters
///
/// NOTE: may not have any trailing/preceding whitespace. To allow trailing characters, see
/// [`timecode_nom`]
///
///```
///use timecode::{
///    unvalidated,
///    parser::{Seperator, UnvalidatedTC},
///};
///
///assert_eq!(
///    unvalidated("01:23:12:22"),
///    Some(UnvalidatedTC {
///        h: 1,
///        m: 23,
///        s: 12,
///        f: 22,
///        seperator: Seperator::Colon,
///    })
///);
///
///assert_eq!(
///    unvalidated("01:23:12;22"),
///    Some(UnvalidatedTC {
///        h: 1,
///        m: 23,
///        s: 12,
///        f: 22,
///        seperator: Seperator::Semicolon,
///    })
///);
///
///assert_eq!(
///    unvalidated("012312:22"),
///    None,
///);
///
///assert_eq!(
///    unvalidated("Not a timecode"),
///    None,
///);
///```
pub fn unvalidated(input: &str) -> Option<UnvalidatedTC> {
    timecode_nom(input)
        .map(|(remaining_input, v)| {
            //TODO: should this allow trailing chars?
            match remaining_input {
                "" => Some(v),
                _ => None,
            }
        })
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_tc() {
        assert!(matches!(timecode_nom("01:23:12;22"), Ok(_)));
        assert!(matches!(unvalidated("01:23:12;22"), Some(_)));
    }

    #[test]
    fn parse_h_too_big() {
        assert!(matches!(timecode_nom("911:00:00:00"), Err(_)));
    }

    #[test]
    fn trailing() {
        assert!(matches!(timecode_nom("01:23:12;22 ok"), Ok(_)));
        assert!(matches!(unvalidated("01:23:12;22 ok"), None));
    }

    #[test]
    fn wrong_sep() {
        assert!(matches!(timecode_nom("123;23;23;00"), Err(_)));
    }
}
