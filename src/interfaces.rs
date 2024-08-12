use std::collections::HashMap;

use rs_plugin_common_interfaces::{provider::{RsProviderEntry, RsProviderEntryType}, request::RsRequestMethod, RsFileType, RsRequest};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::{DEFAULT_CLIENT_ID, DEFAULT_CLIENT_SECRET};


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")] 
pub struct PCloudSettings {
    #[serde(default = "default_client_id")]
    pub client_id: String,
    #[serde(default = "default_client_secret")]
    pub client_secret: String,
}
fn default_client_id() -> String {
    DEFAULT_CLIENT_ID.to_string()
}

fn default_client_secret() -> String {
    DEFAULT_CLIENT_SECRET.to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")] 
pub struct PCloudCredentialsSettings {
    pub hostname: String,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")] 
pub struct TokenResponse {
    pub access_token: String,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")] 
pub struct PCloudErrorResponse {

    pub result: u16,
    
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")] 
pub struct FolderListResponse {
    pub metadata: FolderMetadata,
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")] 
pub struct FolderMetadata {
    //pub path: String,
    pub contents: Vec<PCloudFile>
}


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")] 
pub struct PCloudFile {
    //pub path: String,
    pub name: String,
    pub isfolder: bool,
    pub hash: Option<u64>,
    #[serde(with = "time::serde::rfc2822::option")]
    modified: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc2822::option")]
    created: Option<OffsetDateTime>,
    pub folderid: Option<u64>,
    pub fileid: Option<u64>,
    pub contenttype: Option<String>,
}

impl PCloudFile {

    pub fn id(&self) -> String {
        self.folderid.unwrap_or(self.fileid.unwrap_or_default()).to_string()
    }


}

impl From<PCloudFile> for RsProviderEntry {
    fn from(value: PCloudFile) -> Self {
        RsProviderEntry {
            source: value.id(),
            kind: if value.isfolder { RsProviderEntryType::Directory } else { RsProviderEntryType::Other },
            mimetype: value.contenttype,
            size: None,
            hash: value.hash.map(|f| f.to_string()),
            added: None,
            modified: value.modified.map(|r| r.unix_timestamp() as u64 * 1000),
            created: value.created.map(|r| r.unix_timestamp() * 1000),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PCloudUploadResult {
    pub result: i64,
    pub metadata: Vec<PCloudFile>,
    pub checksums: Vec<PCloudChecksum>,
    pub fileids: Vec<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PCloudChecksum {
    pub sha1: String,
    pub sha256: String,
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PCloudStatResult {
    pub result: i64,
    pub metadata: PCloudFile,
}



#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PCloudLinkResult {
    pub result: i64,
    pub path: String,
    #[serde(with = "time::serde::rfc2822")]
    pub expires: OffsetDateTime,
    pub hosts: Vec<String>,
}

impl From<PCloudLinkResult> for RsRequest {
    fn from(value: PCloudLinkResult) -> Self {
        RsRequest {
            url: format!("https://{}{}",value.hosts.first().unwrap(), value.path),
            permanent: false,
            method: RsRequestMethod::Get,
            ..Default::default()
        }
    }
}