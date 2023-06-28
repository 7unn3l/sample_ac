#![allow(non_snake_case)]
use axum::{response::IntoResponse, routing::post, Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use kube::{
    core::admission::{AdmissionResponse, AdmissionReview},
    CustomResource,
    api::DynamicObject
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, net::SocketAddr, path::PathBuf};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
struct PortEntry {
    name: Option<String>,
    containerPort: Option<u16>,
    hostPort: Option<u16>,
    hostIP: Option<String>,
    protocol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
struct ContainerType {
    name: String,
    image: String,
    ports: Vec<PortEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
struct SecContext{
    privileged: Option<bool>,
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[kube(
    group = "kube.rs",
    version = "v1",
    kind = "BaseAdmissionReview",
    namespaced
)]
pub struct TestSpec {
    containers: Vec<ContainerType>,
}

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();

    let config = RustlsConfig::from_pem_file(
        PathBuf::from("certs/server.crt"),
        PathBuf::from("certs/server.key"),
    )
    .await
    .unwrap();

    let port = 443;
    // Create the TLS configuration
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let app = Router::new().route("/validate", post(validate));

    println!("binding sample AD server to 0.0.0.0:{port}");

    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn get_value(obj: &Value, path: &str) -> Option<Value>{

    let mut obj = obj.clone();
    for part in path.split("."){
        
       match obj{
           Value::Object(m) =>{
               obj = m.get(part).unwrap().clone();
           }
           _ => {panic!("not an object")}
       } 

    }

    return Some(obj)
}

async fn validate(
    Json(mut review): Json<AdmissionReview<DynamicObject>>,
) -> impl IntoResponse {
    // the request might have AdmissgonReview<BaseAdmissionReview>
    // request.request is the "review" portion
    
    println!("validating..");

    let request = review.request.as_ref().unwrap();
    let mut err = String::new();

    // parses the RequestKind portion
    if request.kind.kind != "Pod" {
        write!(err, "Only operating on pods!").ok();
    }

    // in the object portion, there is the spec portion.
    let object = request.object.as_ref().unwrap().data.as_object().unwrap();

    let object = serde_json::Value::Object(object.clone());

    /*

    */

    // check that every speaker can talk

    // is the talk accepted?

    // is the talk in the schedule?

    let mut all_ready = true;

    match get_value(&object,"spec.speaker"){
        Some(speakers) => {
            for speaker in speakers.as_array().unwrap(){
                println!("processing speaker: {}", speaker);

                match get_value(&speaker,"canTalk"){
                    Some(canTalk) => {
                        if canTalk.as_bool().unwrap(){
                            println!("speaker can talk");
                        }
                    }
                    None => {
                        println!("speaker cannot talk");
                        all_ready = false;
                    }
                }
            }
        }
        None => {}

    }

    if !all_ready{
        write!(err, "not all speakers are ready").ok();
    }
   
    /*match get_value(&object,"spec.containers"){
        Some(containers) => {
            for container in containers.as_array().unwrap(){
                match get_value(container,"securityContext.privileged"){
                    Some(privileged) => {
                        if privileged.as_bool().unwrap(){
                            write!(err, "container {} is privileged,", container["name"]).ok();
                        }
                    }
                    None => {}
                }
            }
        }
        None => {
            println!("no containers");
            write!(err, "no containers").ok();
        }
    }
    */
    
    // ready to set response in AdmissionReview
    let mut response = AdmissionResponse::from(request);

    println!("err: {}", err);

    response.allowed = err.is_empty();
    response.result.message = err;

    review.response = Some(response);

    Json(review)
}
