use std::error::Error;
use thiserror::Error;

use serde::{Deserialize, Serialize};

use crate::utils::{self, Location, FID};

pub struct Master {
    pub host: String,
    pub port: Option<u16>,
}

#[derive(Error, Debug)]
enum MasterErrors {
    #[error("Wrong format of string expected 0.0.0.0:3333 for example")]
    WrongFormat,
    #[error("Response StatusCode was not OK see body for error: {0}")]
    InvalidRequest(String),
}

impl Master {
    pub fn to_string(&self) -> String {
        match self.port {
            Some(port) => concat_string!("http://", self.host, ":", port.to_string()),
            _ => concat_string!("http://", self.host, ":9333"),
        }
    }

    /// Creates a master from a string
    /// 
    /// # Example
    /// ```
    /// use rusty_weed::master::Master;
    /// 
    /// let master = Master::from_str("1.1.1.1:9333").unwrap();
    /// ```
    pub fn from_str(s: &str) -> Result<Master, Box<dyn Error>> {
        let mut parts = s.split(":");

        let host: String;
        let port: u16;

        match parts.next() {
            Some(s) => host = s.to_string(),
            None => return Err(Box::new(MasterErrors::WrongFormat)),
        }

        match parts.next() {
            Some(s) => port = s.parse::<u16>()?,
            None => return Err(Box::new(MasterErrors::WrongFormat)),
        }

        Ok(Master {
            host,
            port: Some(port),
        })
    }

    /// Assigns a file id
    pub async fn assign_key(
        &self,
        options: &Option<AssignKeyOptions>,
    ) -> Result<AssignKeyResponse, Box<dyn Error>> {
        let qs_string = serde_qs::to_string(options)?;
        let req = reqwest::get(concat_string!(self.to_string(), "/dir/assign?", qs_string)).await?;

        match req.status() {
            reqwest::StatusCode::OK => Ok(req.json::<AssignKeyResponse>().await?),
            _ => Err(Box::new(MasterErrors::InvalidRequest(req.text().await?))),
        }
    }

    /// Lookup the locations of a volume
    pub async fn lookup_volume(
        &self,
        volume_id: &FID,
        options: &Option<LookupVolumeOptions>,
    ) -> Result<LookupVolumeResponse, Box<dyn Error>> {
        let qs_string = serde_qs::to_string(options)?;

        let req = reqwest::get(concat_string!(
            self.to_string(),
            "/dir/lookup?volumeId=",
            volume_id.volume_id.to_string(),
            "&",
            qs_string
        ))
        .await?;

        match req.status() {
            reqwest::StatusCode::OK => Ok(req.json::<LookupVolumeResponse>().await?),
            _ => Err(Box::new(MasterErrors::InvalidRequest(req.text().await?))),
        }
    }
}

/// Options for the [assign_key](Master::assign_key) function
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssignKeyOptions {
    pub count: Option<u32>,
    pub collection: Option<String>,
    pub data_center: Option<String>,
    pub rack: Option<String>,
    pub data_node: Option<String>,
    pub replication: Option<utils::ReplicationType>,
    pub ttl: Option<utils::TTL>,
    /// If no matching volumes, pre-allocate this number of bytes on disk for new volumes.
    pub preallocate: Option<u64>,
    /// If no matching volumes, create specified number of new volumes.
    /// Default: master preallocateSize
    pub writable_volume_count: Option<u64>,
    /// If you have disks labelled, this must be supplied to specify the disk type to allocate on.
    /// Default: empty
    pub disk: Option<String>,
}

/// Return type of the [assign_key](Master::assign_key) function
#[derive(Deserialize, Debug)]
pub struct AssignKeyResponse {
    pub count: u64,
    pub fid: FID,
    #[serde(flatten)]
    pub location: Location,
}

/// Options for the [lookup_volume](Master::lookup_volume) function
#[derive(Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LookupVolumeOptions {
    pub collection: Option<String>,
    pub file_id: Option<FID>,
    pub read: Option<bool>,
}

/// Return type of the [lookup_volume](Master::lookup_volume) function
#[derive(Deserialize, Debug)]
pub struct LookupVolumeResponse {
    pub locations: Vec<Location>,
}

#[cfg(test)]
mod tests {
    static MASTER_HOST: &str = "localhost";
    static MASTER_PORT: u16 = 8333;

    use crate::utils::FID;

    use super::{AssignKeyResponse, AssignKeyOptions, LookupVolumeOptions, Master};

    #[test]
    fn parse_resp_assign_key() {
        let data = r#"{
            "count": 1,
            "fid":"3,01637037d6",
            "publicUrl":"1.1.1.1:9333",
            "url":"1.2.2.2:3233"
        }"#;

        let parsed = serde_json::from_str::<AssignKeyResponse>(data);
        match parsed {
            Ok(f) => assert_eq!("3,01637037d6", f.fid.to_string().as_str()),
            Err(e) => {
                println!("{}", e);
                panic!("Failed to parse RespAssignKey");
            }
        }
    }

    #[tokio::test]
    async fn call_assign_key() {
        let master = Master {
            host: MASTER_HOST.to_string(),
            port: Some(MASTER_PORT),
        };

        let options: AssignKeyOptions = Default::default();
        let resp = master.assign_key(&Some(options)).await;

        match resp {
            Ok(x) => {
                println!("New assigned file id: {}", x.fid.to_string());
                assert_eq!(1, x.count);
            }
            _ => panic!("failed to assign key"),
        }
    }

    #[tokio::test]
    async fn lookup_volume() {
        let master = Master {
            host: MASTER_HOST.to_string(),
            port: Some(MASTER_PORT),
        };

        let options_assign: AssignKeyOptions = Default::default();
        let resp_assign = master.assign_key(&Some(options_assign)).await;

        let fid: FID;

        match resp_assign {
            Ok(x) => {
                println!("New assigned file id: {}", x.fid.to_string());
                fid = x.fid
            }
            _ => panic!("failed to assign key"),
        }

        let options_lookup: LookupVolumeOptions = Default::default();
        let resp_lookup = master.lookup_volume(&fid, &Some(options_lookup)).await;

        match resp_lookup {
            Ok(x) => {
                assert!(x.locations.len() > 0);
                let location = &x.locations[0];
                println!("New assigned file id: {}", location.public_url);
            }
            _ => panic!("failed to lookup volume"),
        }

    }
}
