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

// impl Serialize for DynFramerate {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let s = self.to_string();
//         serializer.serialize_str(&s)
//     }
// }

impl<FR: Framerate> Serialize for Timecode<FR> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de, FR> Deserialize<'de> for Timecode<FR> 
where FR: Framerate + validate::ValidateableFramerate + ConstFramerate{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl<'de > Deserialize<'de> for Timecode<DynFramerate> 
{
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

    // #[test]
    // fn serialize_fr() {
    //     let fr1: DynFramerate = "29.97".parse().unwrap();
    //     let s1 = serde_json::to_string(&fr1).unwrap();
    //     assert_eq!(s1, "\"29.97\"");
    //
    //     let fr2: DynFramerate = "23.976".parse().unwrap();
    //     let s2 = serde_json::to_string(&fr2).unwrap();
    //     assert_eq!(s2, "\"23.98\"");
    // }

    #[test]
    fn deserialize_fr() {
        let s1 = "\"23.976\"";
        let fr1: DynFramerate = serde_json::from_str(s1).unwrap();
        assert_eq!(fr1, DynFramerate::new_ndf(24));

        let s2 = "\"29.97\"";
        let fr2: DynFramerate = serde_json::from_str(s2).unwrap();
        assert_eq!(fr2, DynFramerate::new_df(30));
    }

    #[test]
    fn serialize_timecode() {
        let s1 = "01:10:00;12";
        let t1: Timecode<DF<30>> = s1.parse().unwrap();
        let ser = serde_json::to_string(&t1).unwrap();
        assert_eq!(ser, format!("\"{s1}\"")); 

        let s2 = "01:10:00:12";
        let t2: Timecode<NDF<30>> =  s2.parse().unwrap();
        let ser2 = serde_json::to_string(&t2).unwrap();
        assert_eq!(ser2, format!("\"{s2}\"")); 
    }
}
