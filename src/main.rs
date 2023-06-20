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
   
    match get_value(&object,"spec.containers"){
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


    /*
    for container in object.spec.containers.iter() {
        println!("processing container {}", container.name);

        /*if container.securityContext.privileged.is_some(){
            if container.securityContext.privileged.unwrap() {
                write!(err, "container {} is privileged\n", container.name).ok();
            }
        }
        */

        for portent in container.ports.iter() {
            match &portent.name {
                Some(pname) => {
                    if pname.len() >= 10 {
                        write!(
                            err,
                            "container {} has portname thats too long: {pname}\n",
                            container.name
                        )
                        .ok();
                    }
                }

                None => {}
            }

            match portent.containerPort {
                Some(port) => {
                    if ports_seen.contains(&port) {
                        write!(
                            err,
                            "port {} of container {} already seen\n",
                            port, container.name
                        )
                        .ok();
                    } else {
                        ports_seen.push(port)
                    }
                }
                None => {}
            }
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
