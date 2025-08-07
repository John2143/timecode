use serde::{Deserialize, Serialize};
use crate::{validate, ConstFramerate, DynFramerate, Framerate, FromStr, Timecode};

impl <'de> Deserialize<'de> for DynFramerate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for DynFramerate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<FR: Framerate> Serialize for Timecode<FR> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for Timecode<DynFramerate> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod test_json{
    use crate::{Timecode, DF, NDF};
    use super::*;

    #[test]
    fn serialize_fr() {
        let fr: DynFramerate = "29.97".parse().unwrap();
        let s = serde_json::to_string(&fr).unwrap();
        assert_eq!(s, "\"29.97\"");
    }

    #[test]
    fn deserialize_fr() {
        let s = "\"29.97\"";
        let fr: DynFramerate = serde_json::from_str(s).unwrap();
        assert_eq!(fr, DynFramerate::new_df(30));
    }

    #[test]
    fn serialize_timecode() {
        let s = "01:10:00;12";
        let t1: Timecode<DF<30>> = s.parse().unwrap();
        let ser = serde_json::to_string(&t1).unwrap();
        assert_eq!(ser, format!("\"{s}\"")); 

        let t2: Timecode<NDF<30>> =  s.parse().unwrap();
        let ser2 = serde_json::to_string(&t2).unwrap();
        assert_eq!(ser2, format!("\"{s}\"")); 
    }
}
