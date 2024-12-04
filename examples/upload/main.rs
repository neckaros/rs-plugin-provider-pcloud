
use extism::{convert::Json, Manifest, PluginBuilder};
use std::path::PathBuf;

use reqwest::Client;
use tokio::fs::File;
use tokio_util::io::ReaderStream;


use rs_plugin_common_interfaces::{provider::{RsProviderAddRequest, RsProviderAddResponse, RsProviderEntry, RsProviderPath}, PluginCredential, RsPluginRequest};

use serde_json::json;


#[tokio::main]
async fn main() {
    extism::set_log_callback(|f| println!("{}",f), "info");
    println!("Hello from an example!");
    let manifest = Manifest::new([PathBuf::from("target/wasm32-unknown-unknown/release/rs_plugin_provider_pcloud.wasm")]).with_allowed_host("*");
            let mut plugin = PluginBuilder::new(manifest)
                .with_wasi(true)
                .build().unwrap();
        




    let request = RsProviderAddRequest {
        root: "/test".to_string(),
        name: "test.rtf".to_string(),
        overwrite: true,
    };

    let call_object: RsPluginRequest<RsProviderAddRequest> = RsPluginRequest {
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

        
    let Json(token) = plugin.call::<Json<RsPluginRequest<RsProviderAddRequest>>, Json<RsProviderAddResponse>>("upload_request", Json(call_object)).unwrap();

    println!("link: {:?}", token);

    let file = File::open("examples/upload/test.rtf").await.unwrap();
    let file_size = file.metadata().await.unwrap().len();
    let stream = ReaderStream::new(file);
    let body = reqwest::Body::wrap_stream(stream);

    let client = Client::new();
    let response = client
        .post(token.request.url)
        .header("Authorization", format!("Bearer {}", "token"))
        .header("Content-Length", file_size)
        .header("Content-Type", "application/rtf")
       //.header("Connection", "keep-alive")
        //.header("Accept-Encoding", "gzip, deflate, br")
        .body(body)
        .send()
        .await.unwrap();

    println!("Upload status: {}", response.status());
    println!("Upload response: {}", response.text().await.unwrap());

    let json_string = r#"{
        "result": 0,
        "metadata": [
            {
                "name": "test.txt",
                "created": "Mon, 12 Aug 2024 07:06:06 +0000",
                "thumb": false,
                "modified": "Mon, 12 Aug 2024 07:06:06 +0000",
                "isfolder": false,
                "fileid": 48896126632,
                "hash": 10638145551977968182,
                "path": "/test/test.txt",
                "category": 4,
                "id": "f48896126632",
                "isshared": false,
                "ismine": true,
                "size": 87389,
                "parentfolderid": 12426517578,
                "contenttype": "text/plain",
                "icon": "document"
            }
        ],
        "checksums": [
            {
                "sha1": "ecf674a59f72531b3d1adb70dfc95bd5fef663ff",
                "sha256": "d78da359fbc11e845839dfd69ba2c6fc7cc1e6b8867ec3173cbe8b652940a831"
            }
        ],
        "fileids": [
            48896126632
        ]
    }"#;

    let call_object: RsPluginRequest<String> = RsPluginRequest {
        request: json_string.to_string(),
        plugin_settings: json!({}),
        credential: Some(PluginCredential {
            password: Some("token".to_string()),
            settings: json!({ 
                "hostname": "eapi.pcloud.com",
            }),
            ..Default::default()
        })
    };


    let source = plugin.call::<Json<RsPluginRequest<String>>, Json<RsProviderEntry>>("upload_response", Json(call_object));

    println!("source: {:?}", source);

}