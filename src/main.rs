#![allow(non_snake_case)]
use axum::{response::IntoResponse, routing::post, Json, Router};
use axum_server::tls_rustls::RustlsConfig;
use kube::{
    core::admission::{AdmissionResponse, AdmissionReview},
    CustomResource
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{fmt::Write, net::SocketAddr, path::PathBuf};

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

async fn validate(
    Json(mut review): Json<AdmissionReview<BaseAdmissionReview>>,
) -> impl IntoResponse {
    // the request might have AdmissgonReview<BaseAdmissionReview>
    // request.request is the "review" portion

    let request = review.request.as_ref().unwrap();
    let mut err = String::new();

    // parses the RequestKind portion
    if request.kind.kind != "Pod" {
        write!(err, "Only operating on pods!").ok();
    }

    // in the object portion, there is the spec portion.
    let object = request.object.as_ref().unwrap();

    let mut ports_seen: Vec<u16> = vec![];

    for container in object.spec.containers.iter() {
        println!("processing container {}", container.name);

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
    // ready to set response in AdmissionReview
    let mut response = AdmissionResponse::from(request);

    response.allowed = err.is_empty();
    response.result.message = err;

    review.response = Some(response);

    Json(review)
}
