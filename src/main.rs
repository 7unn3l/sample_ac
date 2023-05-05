#![allow(unused)]
use axum::{extract, routing::post, Json, Router};
use kube::core::admission::{AdmissionRequest, AdmissionReview};
use kube::CustomResource;
use kube::ResourceExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::iter::Map;


#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
struct PortEntry{
   name : Option<String>,
   containerPort : Option<u16>,
   hostPort : Option<u16>,
   hostIP : Option<String>,
   protocol : Option<String>
    
} 


#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
struct ContainerType {
    name: String,
    image: String,
    ports: Vec<PortEntry> 
}

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[kube(group = "kube.rs", version = "v1", kind = "BaseAdmissionReview", namespaced)]
pub struct TestSpec {
    containers: Vec<ContainerType>,
}

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    let app = Router::new().route("/validate", post(validate));

    println!("binding sample AD server to 0.0.0.0:8080");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn validate(Json(request): Json<AdmissionReview<BaseAdmissionReview>>) -> String {
    // review -> request -> response -> review
    // review -> rewquest == .try_into().unwrap()
    // requets -> response -> AdmissionResponse::from(&request)
    // response -> review == .into_review()
   
    // the request might have AdmissionReview<BaseAdmissionReview> 
    // request.request is the "review" portion
    let request = request.request.unwrap(); 

    // parses the RequestKind portion 
    if (request.kind.kind != "Pod"){
        return "Only operating on pods!".to_string();
    }
    
    // in the object portion, there is the spec portion.
    let spec = request.object.unwrap().spec;
  
    let mut ports_seen:Vec<u16> = vec![];
    for container in spec.containers.iter(){
        println!("processing container {}",container.name);

        for portent in container.ports.iter(){

            match &portent.name{
                Some(pname) => {
                    if pname.len() >= 10{
                        println!("container {} has portname thats too long: {}",container.name,pname);
                    }
                },

                None => {}
            } 
            

        }
    }

    "Ok".to_string()
}
