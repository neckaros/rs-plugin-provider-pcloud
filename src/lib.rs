use std::collections::{BTreeMap, HashMap};

use extism_pdk::{Error, WithReturnCode};
use extism_pdk::{error, http, info, log, plugin_fn, warn, FnResult, HttpRequest, Json};
use interfaces::{FolderListResponse, PCloudCredentialsSettings, PCloudErrorResponse, PCloudFile, PCloudLinkResult, PCloudSettings, PCloudStatResult, PCloudUploadResult, TokenResponse};
use rs_plugin_common_interfaces::provider::{RsProviderAddRequest, RsProviderAddResponse, RsProviderEntry, RsProviderPath};
use rs_plugin_common_interfaces::request::RsRequestMethod;
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

pub fn get_url(path: String, credential: &PluginCredential, params: HashMap<&str, String>) -> FnResult<RsRequest> {
    let pcloud_credential: PCloudCredentialsSettings = parse_credentials_settings(credential.settings.clone())?;

    let params_string = params.into_iter().map(|(key, value)| format!("{}={}", key, encode(&value))).collect::<Vec<_>>().join("&");
    //info!("{}?apikey={}&{}", url, token, params_string);
    Ok(RsRequest {
        url: format!("https://{}{}?{}", pcloud_credential.hostname, path, params_string),
        headers: Some(vec![("authorization".to_owned(), format!("Bearer {}", credential.password.clone().unwrap_or_default()))]),
        method: RsRequestMethod::Post,
        ..Default::default()
    })
}

#[plugin_fn]
pub fn upload_request(Json(settings): Json<RsPluginRequest<RsProviderAddRequest>>) -> FnResult<Json<RsProviderAddResponse>> {
    let request: RsRequest = get_url(
        "/uploadfile".to_string(), 
        &settings.credential.ok_or(Error::msg("No code provided"))?, 
        HashMap::from([("filename", settings.request.name), 
        ("path", settings.request.root), 
        ("nopartial", "1".to_string()), 
        ("renameifexists", (if settings.request.overwrite { "0" } else { "1"}).to_string())]))?;
       Ok(Json(
            RsProviderAddResponse { request, multipart: None, source: None, packets: None }
        ))
}


#[plugin_fn]
pub fn upload_response(Json(settings): Json<RsPluginRequest<String>>) -> FnResult<Json<RsProviderEntry>> {
    let response = serde_json::from_str::<PCloudUploadResult>(&settings.request)?;
    let file = response.metadata.first().ok_or(Error::msg("unable to get file from response"))?.clone();
    Ok(Json(file.into()))
}

#[plugin_fn]
pub fn download_request(Json(request): Json<RsPluginRequest<RsProviderPath>>) -> FnResult<Json<RsRequest>> {

    let credentials = request.credential.ok_or(Error::msg("Token not provided"))?;
    let token = credentials.password.ok_or(Error::msg("Token not provided"))?;
    let pcloud_credential = parse_credentials_settings(credentials.settings)?;


    let url = format!("https://{}/getfilelink?fileid={}", pcloud_credential.hostname, request.request.source);
    let req = HttpRequest {
        url,
        headers: BTreeMap::from([("authorization".to_owned(), format!("Bearer {}", token))]),
        method: Some("GET".into()),
    };

    let res = http::request::<()>(&req, None)?;
    if let Ok(json) = res.json::<PCloudLinkResult>() {
        let result: RsRequest = json.into();
        Ok(Json(result))
    } else if  let Ok(json) = res.json::<PCloudErrorResponse>() {
        error!("request error: {:?}", json);
        Err(WithReturnCode::new(Error::msg(format!("Error fetching folder content: {}", json.error)), 500))
    } else {
        info!("Content: {:?}", String::from_utf8_lossy(&res.body()));
        Err(WithReturnCode::new(Error::msg(format!("Error parsing result: {:?}", res.json::<PCloudLinkResult>())), 500))
    }
}


#[plugin_fn]
pub fn remove_file(Json(request): Json<RsPluginRequest<RsProviderPath>>) -> FnResult<()> {

    let credentials = request.credential.ok_or(Error::msg("Token not provided"))?;
    let token = credentials.password.ok_or(Error::msg("Token not provided"))?;
    let pcloud_credential = parse_credentials_settings(credentials.settings)?;


    let url = format!("https://{}/deletefile?fileid={}", pcloud_credential.hostname, request.request.source);
    let req = HttpRequest {
        url,
        headers: BTreeMap::from([("authorization".to_owned(), format!("Bearer {}", token))]),
        method: Some("GET".into()),
    };

    let res = http::request::<()>(&req, None)?;
    if let Ok(json) = res.json::<PCloudStatResult>() {
        
        Ok(())
    } else if  let Ok(json) = res.json::<PCloudErrorResponse>() {
        error!("request error: {:?}", json);
        Err(WithReturnCode::new(Error::msg(format!("Error deleting file: {}", json.error)), 500))
    } else {
        info!("Content: {:?}", String::from_utf8_lossy(&res.body()));
        Err(WithReturnCode::new(Error::msg(format!("Error parsing result: {:?}", res.json::<PCloudStatResult>())), 500))
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
pub fn list_path(Json(request): Json<RsPluginRequest<RsProviderPath>>) -> FnResult<Json<Vec<RsProviderEntry>>> {
    let credentials = request.credential.ok_or(Error::msg("Token not provided"))?;
    let token = credentials.password.ok_or(Error::msg("Token not provided"))?;
    let pcloud_credential = parse_credentials_settings(credentials.settings)?;
    //let plugin_settings = settings_from_value(request.plugin_settings)?;

    let paths = list_path_internal(&pcloud_credential.hostname, &token, &request.request.source)?;
    
    let entries = paths.into_iter().map(|p| p.into()).collect::<Vec<RsProviderEntry>>();
    Ok(Json(entries))
}
fn list_path_internal(hostname: &str, token: &str, path: &str) -> FnResult<Vec<PCloudFile>> {
    let url = format!("https://{}/listfolder?folderid={}", hostname, path);
    let req = HttpRequest {
        url,
        headers: BTreeMap::from([("authorization".to_owned(), format!("Bearer {}", token))]),
        method: Some("GET".into()),
    };


    let res = http::request::<()>(&req, None);

    if let Ok(res) = res {
        if let Ok(json) = res.json::<FolderListResponse>() {
            Ok(json.metadata.contents)
        } else if  let Ok(json) = res.json::<PCloudErrorResponse>() {
            error!("request error: {:?}", json);
            Err(WithReturnCode::new(Error::msg(format!("Error fetching folder content: {}", json.error)), 500))
        } else {
            info!("Content: {:?}", String::from_utf8_lossy(&res.body()));
            Err(WithReturnCode::new(Error::msg(format!("Error parsing result: {:?}", res.json::<FolderListResponse>())), 500))
        }
   

    } else {
        Err(WithReturnCode::new(Error::msg(format!("Error getting token")), 500))
    }


}

#[plugin_fn]
pub fn file_info(Json(request): Json<RsPluginRequest<RsProviderPath>>) -> FnResult<Json<RsProviderEntry>> {

    let credentials = request.credential.ok_or(Error::msg("Token not provided"))?;
    let token = credentials.password.ok_or(Error::msg("Token not provided"))?;
    let pcloud_credential = parse_credentials_settings(credentials.settings)?;


    let url = format!("https://{}/stat?fileid={}", pcloud_credential.hostname, request.request.source);
    let req = HttpRequest {
        url,
        headers: BTreeMap::from([("authorization".to_owned(), format!("Bearer {}", token))]),
        method: Some("GET".into()),
    };

    let res = http::request::<()>(&req, None)?;
    if let Ok(json) = res.json::<PCloudStatResult>() {
        let result: RsProviderEntry = json.metadata.into();
        Ok(Json(result))
    } else if  let Ok(json) = res.json::<PCloudErrorResponse>() {
        error!("request error: {:?}", json);
        Err(WithReturnCode::new(Error::msg(format!("Error fetching folder content: {}", json.error)), 500))
    } else {
        info!("Content: {:?}", String::from_utf8_lossy(&res.body()));
        Err(WithReturnCode::new(Error::msg(format!("Error parsing result: {:?}", res.json::<PCloudStatResult>())), 500))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth() -> Result<(), Box<dyn std::error::Error>> {
        let folders = list_path_internal("eapi.pcloud.com", "xxx", "/");

        Ok(())
    }

    #[test]
    fn test_url() -> Result<(), Box<dyn std::error::Error>> {
        let credential = "{\"kind\": \"oauth\", \"password\": \"pwd\", \"settings\": {\"location\": 1}}";
        let credential: PluginCredential = serde_json::from_str(credential)?;

        let request = get_url("/testpath".to_owned(), &credential, HashMap::from([("test", "toto".to_owned())])).unwrap();
        assert_eq!("https://api.pcloud.com/testpath?test=toto", request.url);
        assert_eq!(Some(vec![("authorization".to_owned(), "Bearer pwd".to_owned())]), request.headers);
        println!("urk: {:?}", request);
        Ok(())
    }
}
