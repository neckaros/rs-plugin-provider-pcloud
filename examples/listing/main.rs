
use extism::{convert::Json, Manifest, PluginBuilder, Wasm};
use std::{collections::HashMap, fs::read_dir, path::PathBuf, sync::Mutex};

use rs_plugin_common_interfaces::{PluginCredential, PluginInformation, RsPluginRequest};

use serde_json::{json, Value};


fn main() {
    extism::set_log_callback(|f| println!("{}",f), "info");
    println!("Hello from an example!");
    let manifest = Manifest::new([PathBuf::from("target\\wasm32-unknown-unknown\\release\\rs_plugin_provider_pcloud.wasm")]).with_allowed_host("*");
            let mut plugin = PluginBuilder::new(manifest)
                .with_wasi(true)
                .build().unwrap();
        


    let call_object: RsPluginRequest<String> = RsPluginRequest {
        request,
        plugin_settings: json!({}),
        credential: Some(PluginCredential {
            password: Some("token".to_string()),
            ..Default::default()
        })
    };
        
    let token = plugin.call::<Json<RsPluginRequest<HashMap<String, String>>>, Json<PluginCredential>>("list_path", Json(call_object));

    println!("token: {:?}", token);

}