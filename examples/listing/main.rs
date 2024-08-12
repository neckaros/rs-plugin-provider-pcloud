
use extism::{convert::Json, Manifest, PluginBuilder};
use std::path::PathBuf;

use rs_plugin_common_interfaces::{provider::{RsProviderEntry, RsProviderPath}, PluginCredential, RsPluginRequest, RsRequest};

use serde_json::json;


fn main() {
    extism::set_log_callback(|f| println!("{}",f), "info");
    println!("Hello from an example!");
    let manifest = Manifest::new([PathBuf::from("target/wasm32-unknown-unknown/release/rs_plugin_provider_pcloud.wasm")]).with_allowed_host("*");
            let mut plugin = PluginBuilder::new(manifest)
                .with_wasi(true)
                .build().unwrap();
        




    let request = RsProviderPath {
        root: Some(1784196375.to_string()),
        source: 1784196375.to_string(),
    };

    let call_object: RsPluginRequest<RsProviderPath> = RsPluginRequest {
        request,
        plugin_settings: json!({}),
        credential: Some(PluginCredential {
            password: Some("token".to_string()),
            settings: json!({ 
                "hostname": "eapi.pcloud.com",
                
            }),
            ..Default::default()
        })
    };

        
    let list = plugin.call::<Json<RsPluginRequest<RsProviderPath>>, Json<Vec<RsProviderEntry>>>("list_path", Json(call_object));

    //println!("list: {:?}", list);



    let request = RsProviderPath {
        root: Some(1784196375.to_string()),
        source: "48896126632".to_string(),
    };

    let call_object: RsPluginRequest<RsProviderPath> = RsPluginRequest {
        request,
        plugin_settings: json!({}),
        credential: Some(PluginCredential {
            password: Some("token".to_string()),
            settings: json!({ 
                "hostname": "eapi.pcloud.com",
                
            }),
            ..Default::default()
        })
    };

    let file = plugin.call::<Json<RsPluginRequest<RsProviderPath>>, Json<RsProviderEntry>>("file_info", Json(call_object.clone()));

    println!("file: {:?}", file);


    let file = plugin.call::<Json<RsPluginRequest<RsProviderPath>>, Json<RsRequest>>("download_request", Json(call_object.clone()));

    println!("link: {:?}", file);


    let file = plugin.call::<Json<RsPluginRequest<RsProviderPath>>, ()>("remove_file", Json(call_object));

    println!("link: {:?}", file);

}