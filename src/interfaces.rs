use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub locationid: String,
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