use std::{error::Error, fmt};

use serde::{
    de::{self, Visitor},
    Serialize, Serializer, Deserialize, Deserializer,
};
use thiserror::Error;

/// SeaweedFS only allows a max replication of 2 per type
/// so we use the enum to implement this limit
#[derive(Debug)]
pub enum ReplicationValues {
    OneReplica,
    TwoReplicas,
}

impl ReplicationValues {
    pub fn to_string(&self) -> String {
        match self {
            Self::OneReplica => "1".to_string(),
            Self::TwoReplicas => "2".to_string(),
        }
    }
}

/// Replication factor for volumes
/// for example 100 means 1 replica in another data center
#[derive(Debug)]
pub struct ReplicationType {
    data_center: Option<ReplicationValues>,
    other_rack: Option<ReplicationValues>,
    same_rack: Option<ReplicationValues>,
}

impl ReplicationType {
    pub fn to_string(&self) -> String {
        let mut s = String::new();

        match &self.data_center {
            Some(val) => s = concat_string!(s, val.to_string()),
            _ => s += "0",
        }

        match &self.other_rack {
            Some(val) => s = concat_string!(s, val.to_string()),
            _ => s += "0",
        }

        match &self.same_rack {
            Some(val) => s = concat_string!(s, val.to_string()),
            _ => s += "0",
        }

        s
    }
}

impl Serialize for ReplicationType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

/// Units for TTL for requesting a file key
#[derive(Debug)]
pub enum TTLUnits {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl TTLUnits {
    pub fn to_string(&self) -> String {
        match self {
            Self::Minute => "m".to_string(),
            Self::Hour => "h".to_string(),
            Self::Day => "d".to_string(),
            Self::Week => "w".to_string(),
            Self::Month => "M".to_string(),
            Self::Year => "y".to_string(),
        }
    }
}

/// Time to live option struct for assigning a file id
#[derive(Debug)]
pub struct TTL {
    pub unit: TTLUnits,
    pub value: u32,
}

impl TTL {
    pub fn to_string(&self) -> String {
        concat_string!(self.unit.to_string(), self.value.to_string())
    }
}

impl Serialize for TTL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[derive(Error, Debug)]
enum FIDErrors {
    #[error("Missing formatted volume id")]
    MissingVolumeId,
    #[error("Missing formatted file string")]
    MissingFileString,
}

/// Representation of a SeaweedFS file id (3,32834855_1 for example)
#[derive(Debug)]
pub struct FID {
    pub volume_id: u32,
    pub file_string: String,
    pub count: Option<u64>,
}

impl FID {
    pub fn to_string(&self) -> String {
        let tmp = concat_string!(self.volume_id.to_string(), ",", self.file_string);

        match self.count {
            Some(count) => concat_string!(tmp, "_", count.to_string()),
            _ => tmp,
        }
    }

    pub fn from_string(s: &str) -> Result<FID, Box<dyn Error>> {
        let mut parts = s.split(",");

        let volume_id: u32;
        let file_string;
        let mut count = None;

        match parts.next() {
            Some(s) => volume_id = s.parse::<u32>()?,
            None => return Err(Box::new(FIDErrors::MissingVolumeId)),
        }

        match parts.next() {
            Some(s) => {
                let mut count_parts = s.split("_");
                match count_parts.next() {
                    Some(s) => file_string = s.to_string(),
                    None => return Err(Box::new(FIDErrors::MissingFileString)),
                }

                match count_parts.next() {
                    Some(s) => count = Some(s.parse::<u64>()?),
                    None => (),
                }
            },
            None => return Err(Box::new(FIDErrors::MissingFileString)),
        }

        
        Ok(FID {
            volume_id,
            file_string,
            count,
        })
    }
}

impl<'de> Deserialize<'de> for FID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FIDVisitor;
        impl<'de> Visitor<'de> for FIDVisitor {
            type Value = FID;
        
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string like 3,344924_0")
            }
        
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let res = FID::from_string(value);
        
                match res {
                    Ok(fid) => Ok(fid),
                    Err(err) => Err(E::custom(err.to_string()))
                }
            }
        }

        deserializer.deserialize_str(FIDVisitor)
    }
}

impl Serialize for FID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

/// Location strings for volume lookup
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub public_url: String,
    pub url: String,
}


#[cfg(test)]
mod tests {
    use crate::utils::FID;

    #[test]
    fn check_fid_parsing() {
        let fid_str = "3,5442434343_2";
        let fid = FID::from_string(fid_str);

        match fid {
            Ok(f) => assert_eq!(fid_str, f.to_string().as_str()),
            _ => panic!("Failed to parse fid")
        }
    }
}