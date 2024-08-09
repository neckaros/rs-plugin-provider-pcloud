use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
#[serde(rename_all = "snake_case")] 
pub struct TokenResponse {
    pub access_token: String,
}
