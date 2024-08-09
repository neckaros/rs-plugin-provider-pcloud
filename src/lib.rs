use std::collections::{BTreeMap, HashMap};

use extism_pdk::{Error, WithReturnCode};
use extism_pdk::{error, http, info, log, plugin_fn, warn, FnResult, HttpRequest, Json};
use interfaces::{PCloudCredentialsSettings, PCloudErrorResponse, PCloudSettings, TokenResponse};
use rs_plugin_common_interfaces::{CredentialType, CustomParam, CustomParamTypes, PluginCredential, PluginInformation, PluginType, RsRequest, RsPluginRequest};
use serde::Deserialize;
use serde_json::{json, Value};
use urlencoding::encode;
pub mod interfaces;
#[plugin_fn]
pub fn infos() -> FnResult<Json<PluginInformation>> {
    Ok(Json(
        infos_internal()
    ))
}

pub static DEFAULT_CLIENT_ID: &str = "4zSGTdUSUBV";
pub static DEFAULT_CLIENT_SECRET: &str = "uoUSvu8YcxXzb2iRGssdhyuFr9sk";

pub fn infos_internal() -> PluginInformation {
    warn!("VALUESTY");
    PluginInformation { 
        name: "pcloud".into(), 
        capabilities: vec![PluginType::Provider], 
        version: 1, 
        interface_version: 1, 
        publisher: "neckaros".into(), 
        description: "PCloud provider".into(), 
        settings: vec![
            CustomParam { name: "client_id".to_owned(), param: CustomParamTypes::Text(None), description: Some("You can provide your own clientid and client secret if you wish".to_owned()), required:false },
            CustomParam { name: "client_secret".to_owned(), param: CustomParamTypes::Text(None), description: Some("You can provide your own clientid and client secret if you wish".to_owned()), required:false }
        ],
        credential_kind: Some(CredentialType::Oauth { url: format!("https://my.pcloud.com/oauth2/authorize?response_type=code&client_id={}&state=#state#&redirect_uri=#redirecturi#", DEFAULT_CLIENT_ID) }), 
        ..Default::default() }

}

pub fn settings_from_value(value: Value)-> FnResult<PCloudSettings> {
    let user_settings: PCloudSettings = serde_json::from_value(value).map_err(|e| {
        error!("Error deserializing settings: {:?}", e);
        Error::msg("Unable to deserialize settings")
})?;

    Ok(user_settings)
}

pub fn parse_credentials_settings(value: Value)-> FnResult<PCloudCredentialsSettings> {
    let cred_settings: PCloudCredentialsSettings = serde_json::from_value(value).map_err(|e| {
        error!("Error deserializing credentials settings: {:?}", e);
        Error::msg("Unable to deserialize credentials settings")
})?;

    Ok(cred_settings)
}

pub fn get_oauth_url(settings: PCloudSettings) -> String {
    format!("https://my.pcloud.com/oauth2/authorize?response_type=code&client_id={}&state=#state#&redirect_uri=#redirecturi#", settings.client_id)
}

pub fn get_base_url(credential: &PluginCredential) -> String {
    let location = credential.settings["location"].as_u64().unwrap_or(1);
    if location == 1 { "api.pcloud.com".to_owned() } else { "eapi.pcloud.com".to_owned() }
}

pub fn get_url(path: String, credential: &PluginCredential, params: HashMap<&str, String>) -> RsRequest {
    let base = get_base_url(&credential);

    let params_string = params.into_iter().map(|(key, value)| format!("{}={}", key, encode(&value))).collect::<Vec<_>>().join("&");
    //info!("{}?apikey={}&{}", url, token, params_string);
    RsRequest {
        url: format!("https://{}{}?{}", base, path, params_string),
        headers: Some(vec![("authorization".to_owned(), format!("Bearer {}", credential.password.clone().unwrap_or_default()))]),
        ..Default::default()
    }
}

#[plugin_fn]
pub fn exchange_token(Json(settings): Json<RsPluginRequest<HashMap<String, String>>>) -> FnResult<Json<PluginCredential>> {
    let code = settings.request.get("code").ok_or(Error::msg("No code provided"))?;
    let hostname = settings.request.get("hostname").ok_or(Error::msg("No hostname provided"))?;
    let locationid = settings.request.get("locationid").ok_or(Error::msg("No hostname provided"))?;
    let plugin_settings = settings_from_value(settings.plugin_settings)?;

    let cred_settings = PCloudCredentialsSettings {
        hostname: hostname.to_string(),
        locationid: locationid.to_string()
    };

    info!("Token Code: {:?}", code);
    let result = exchange_token_internal(hostname, code, DEFAULT_CLIENT_ID, DEFAULT_CLIENT_SECRET)?;

    let cred = PluginCredential {
        kind: CredentialType::Oauth { url: get_oauth_url(plugin_settings) },
        login: None,
        password: Some(result.access_token),
        settings: json!(cred_settings),
        refresh_token: None,
        expires: None,
        ..Default::default()
    };
    Ok(Json(cred))
}

fn exchange_token_internal(hostname: &str, code: &str, client_id: &str, client_secret: &str) -> FnResult<TokenResponse> {
    let url = format!("https://{}/oauth2_token?client_id={}&client_secret={}&code={}", hostname, client_id, client_secret, code);
    warn!("TRYING {}", url);

    let req = HttpRequest {
        url,
        headers: Default::default(),
        method: Some("GET".into()),
    };
    warn!("request done");


    let res = http::request::<()>(&req, None);

    warn!("request result"); 
    if let Ok(res) = res {
        warn!("request result: {}", res.status_code());
        if let Ok(json) = res.json::<TokenResponse>() {
            info!("request result: {:?}", json);
            Ok(json)
        } else if  let Ok(json) = res.json::<PCloudErrorResponse>() {
            error!("request error: {:?}", json);
            Err(WithReturnCode::new(Error::msg(format!("Error getting token: {}", json.error)), 500))
        } else {
            Err(WithReturnCode::new(Error::msg(format!("Error getting token")), 500))
        }
   

    } else {
        Err(WithReturnCode::new(Error::msg(format!("Error getting token")), 500))
    }


}



#[plugin_fn]
pub fn list_path(Json(request): Json<RsPluginRequest<String>>) -> FnResult<Json<Vec<String>>> {
    let token = request.credential.ok_or(Error::msg("Token not provided"))?.password.ok_or(Error::msg("Token not provided"))?;
    let plugin_settings = settings_from_value(request.plugin_settings)?;


    Ok(Json(vec![]))
}
fn folder_listing_internal(hostname: &str, token: &str, path: &str) -> FnResult<TokenResponse> {
    let url = format!("https://{}/listfolder?path={}", hostname, path);
    warn!("TRYING {}", url);

    let req = HttpRequest {
        url,
        headers: BTreeMap::from([("authorization".to_owned(), format!("Bearer {}", token))]),
        method: Some("GET".into()),
    };
    warn!("request done");


    let res = http::request::<()>(&req, None);

    warn!("request result"); 
    if let Ok(res) = res {
        warn!("request result: {}", res.status_code());
        if let Ok(json) = res.json::<TokenResponse>() {
            info!("request result: {:?}", json);
            Ok(json)
        } else if  let Ok(json) = res.json::<PCloudErrorResponse>() {
            error!("request error: {:?}", json);
            Err(WithReturnCode::new(Error::msg(format!("Error getting token: {}", json.error)), 500))
        } else {
            Err(WithReturnCode::new(Error::msg(format!("Error getting token")), 500))
        }
   

    } else {
        Err(WithReturnCode::new(Error::msg(format!("Error getting token")), 500))
    }


}

pub fn refresh_token(left: usize, right: usize) -> usize {
    left + right
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth() -> Result<(), Box<dyn std::error::Error>> {
        

        Ok(())
    }

    #[test]
    fn test_url() -> Result<(), Box<dyn std::error::Error>> {
        let credential = "{\"kind\": \"oauth\", \"password\": \"pwd\", \"settings\": {\"location\": 1}}";
        let credential: PluginCredential = serde_json::from_str(credential)?;

        let request = get_url("/testpath".to_owned(), &credential, HashMap::from([("test", "toto".to_owned())]));
        assert_eq!("https://api.pcloud.com/testpath?test=toto", request.url);
        assert_eq!(Some(vec![("authorization".to_owned(), "Bearer pwd".to_owned())]), request.headers);
        println!("urk: {:?}", request);
        Ok(())
    }
}
