use std::collections::HashMap;

use rs_plugin_common_interfaces::{CredentialType, CustomParam, CustomParamTypes, PluginCredential, PluginInformation, PluginType, RsRequest};
use urlencoding::encode;


pub fn infos() -> PluginInformation {
    PluginInformation { 
        name: "pcloud".into(), 
        capabilities: vec![PluginType::Lookup, PluginType::Request], 
        version: 1, 
        interface_version: 1, 
        publisher: "neckaros".into(), 
        description: "PCloud provider".into(), 
        settings: vec![
            CustomParam { name: "client_id".to_owned(), param: CustomParamTypes::Text(None), description: Some("You can provide your own clientid and client secret if you wish".to_owned()), required:false },
            CustomParam { name: "client_secret".to_owned(), param: CustomParamTypes::Text(None), description: Some("You can provide your own clientid and client secret if you wish".to_owned()), required:false }
        ],
        credential_kind: Some(CredentialType::Oauth { url: "https://my.pcloud.com/oauth2/authorize?response_type=code&client_id=${clientid}&state=&redirect_uri=#redirecturi#".to_owned() }), 
        ..Default::default() }

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





pub fn refresh_token(left: usize, right: usize) -> usize {
    left + right
}


pub fn oauth_exchange_token(kind: CredentialType, code: String, redirect: String) -> PluginCredential {


    PluginCredential {
        kind,
        login: todo!(),
        password: todo!(),
        settings: todo!(),
        user_ref: todo!(),
        refresh_token: todo!(),
        expires: todo!(),
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

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
