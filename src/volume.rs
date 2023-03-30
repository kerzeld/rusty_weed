use std::error::Error;

use bytes::Bytes;
use reqwest::Response;
use serde::{Serialize, Serializer, Deserialize};
use thiserror::Error;

use crate::utils::FID;

pub struct Volume {
    pub host: String,
    pub port: Option<u16>,
}

#[derive(Error, Debug)]
enum VolumeErrors {
    #[error("Wrong format of string expected 0.0.0.0:3333 for example")]
    WrongFormat,
    #[error("Response StatusCode was not CREATED 201 see body for error: {0}")]
    NotCreated(String),
    #[error("Response StatusCode was not OK 200 see body for error: {0}")]
    InvalidRequest(String),
}

impl Volume {
    pub fn to_string(&self) -> String {
        match self.port {
            Some(port) => concat_string!("http://", self.host, ":", port.to_string()),
            _ => concat_string!("http://", self.host, ":9333"),
        }
    }

    pub fn from_str(s: &str) -> Result<Volume, Box<dyn Error>> {
        let mut parts = s.split(":");

        let host: String;
        let port: u16;

        match parts.next() {
            Some(s) => host = s.to_string(),
            None => return Err(Box::new(VolumeErrors::WrongFormat)),
        }

        match parts.next() {
            Some(s) => port = s.parse::<u16>()?,
            None => return Err(Box::new(VolumeErrors::WrongFormat)),
        }

        Ok(Volume {
            host,
            port: Some(port),
        })
    }

    pub async fn get_file_response(
        &self,
        fid: &FID,
        options: &Option<GetFileOptions>,
    ) -> Result<Response, Box<dyn Error>> {
        let qs_string = serde_qs::to_string(options)?;

        let client = reqwest::Client::builder().gzip(true).build()?;

        let req = client
            .get(concat_string!(
                self.to_string(),
                "/",
                fid.to_string(),
                "?",
                qs_string
            ))
            .send()
            .await?;

        match req.status() {
            reqwest::StatusCode::OK => Ok(req),
            _ => Err(Box::new(VolumeErrors::InvalidRequest(req.text().await?))),
        }
    }

    pub async fn get_file_bytes(
        &self,
        fid: &FID,
        options: &Option<GetFileOptions>,
    ) -> Result<Bytes, Box<dyn Error>> {
        let qs_string = serde_qs::to_string(options)?;

        let client = reqwest::Client::builder().gzip(true).build()?;

        let req = client
            .get(concat_string!(
                self.to_string(),
                "/",
                fid.to_string(),
                "?",
                qs_string
            ))
            .send()
            .await?;

        match req.status() {
            reqwest::StatusCode::OK => Ok(req.bytes().await?),
            _ => Err(Box::new(VolumeErrors::InvalidRequest(req.text().await?))),
        }
    }

    pub async fn delete_file(
        &self,
        fid: &FID,
        options: &Option<GetFileOptions>,
    ) -> Result<Bytes, Box<dyn Error>> {
        let qs_string = serde_qs::to_string(options)?;

        let client = reqwest::Client::builder().gzip(true).build()?;

        let req = client
            .get(concat_string!(
                self.to_string(),
                "/",
                fid.to_string(),
                "?",
                qs_string
            ))
            .send()
            .await?;

        match req.status() {
            reqwest::StatusCode::OK => Ok(req.bytes().await?),
            _ => Err(Box::new(VolumeErrors::InvalidRequest(req.text().await?))),
        }
    }

    pub async fn upload_file_bytes(
        &self,
        fid: &FID,
        data: &Bytes,
        options: &Option<UploadFileOptions>,
    ) -> Result<UploadBytes, Box<dyn Error>> {
        let qs_string = serde_qs::to_string(options)?;

        let client = reqwest::Client::builder().build()?;

        let req = client
            .put(concat_string!(
                self.to_string(),
                "/",
                fid.to_string(),
                "?",
                qs_string
            ))
            .body(data.clone())

            .send()
            .await?;


        match req.status() {
            reqwest::StatusCode::CREATED => Ok(req.json::<UploadBytes>().await?),
            _ => Err(Box::new(VolumeErrors::NotCreated(req.text().await?))),
        }
    }
}

#[derive(Debug)]
pub enum GetFileModes {
    Fit,
    Fill,
}

impl Serialize for GetFileModes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            GetFileModes::Fit => serializer.serialize_unit_variant("Fit", 0, "fit"),
            GetFileModes::Fill => serializer.serialize_unit_variant("Fill", 1, "fill"),
        }
    }
}

#[derive(Serialize, Debug, Default)]
pub struct GetFileOptions {
    #[serde(rename = "readDeleted")]
    pub read_deleted: Option<bool>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub mode: Option<GetFileModes>,
    pub crop_x1: Option<u32>,
    pub crop_x2: Option<u32>,
    pub crop_y1: Option<u32>,
    pub crop_y2: Option<u32>,
}

fn serialize_replicated<S>(value: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(x) => match x {
            true => serializer.serialize_str("replicate"),
            false => serializer.serialize_none(),
        },
        None => serializer.serialize_none(),
    }
}

#[derive(Serialize, Debug, Default)]
pub struct UploadFileOptions {
    #[serde(rename = "type")]
    #[serde(serialize_with = "serialize_replicated")]
    pub replicated: Option<bool>,
    /// modification timestamp in epoch seconds
    pub ts: Option<u64>,
    /// content is a chunk manifest file
    pub cm: Option<bool>
}

#[derive(Deserialize, Debug, Default)]
pub struct UploadBytes {
    pub size: usize
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::master::{Master, AssignKeyOptions};

    use crate::utils::FID;
    use crate::volume::{Volume};

    use super::UploadFileOptions;

    static MASTER_HOST: &str = "localhost";
    static MASTER_PORT: u16 = 8333;

    #[test]
    fn serialize_replicated() {
        let data = UploadFileOptions {
            replicated: Some(true),
            ..Default::default()
        };
        let qs_string = serde_qs::to_string(&data);

        match qs_string {
            Ok(st) => assert_eq!("type=replicate", st),
            _ => (),
        }
    }

    #[tokio::test]
    async fn upload_download_bytes() {
        let master = Master {
            host: MASTER_HOST.to_string(),
            port: Some(MASTER_PORT),
        };

        let options: AssignKeyOptions = Default::default();
        let master_resp = master.assign_key(&Some(options)).await;

        let fid: FID;
        let volume: Volume;
        match master_resp {
            Ok(x) => {
                println!("Address {}", x.location.url);
                volume = Volume::from_str(&x.location.url).unwrap();
                fid = x.fid;
            }
            _ => panic!("failed to assign key"),
        }

        let data = Bytes::from("Hello World!");
        let file_resp = volume.upload_file_bytes(&fid, &data, &None).await;

        match file_resp {
            Ok(x) => {
                println!("Body length: {}", data.len());
                assert_eq!(data.len(), x.size);
            }
            Err(err) => {
                println!("{}", err);
                panic!("failed to upload file");
            },
        }

        let down_resp = volume.get_file_bytes(&fid, &None).await;

        match down_resp {
            Ok(x) => {
                assert_eq!(String::from_utf8(data.clone().into()).unwrap(), String::from_utf8(x.clone().into()).unwrap())
            }
            Err(err) => {
                println!("{}", err);
                panic!("failed to download file");
            },
        }

    }
}