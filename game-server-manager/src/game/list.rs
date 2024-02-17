use crate::{config, ServiceKey};
use actix_web::{error, get, HttpResponse};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct Container {
    command: String,
    created_at: String,
    #[serde(rename = "ID")]
    id: String,
    image: String,
    labels: String,
    local_volumes: String,
    mounts: String,
    names: String,
    networks: String,
    ports: String,
    running_for: String,
    size: String,
    state: ContainerState,
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ContainerState {
    Created,
    Restarting,
    Running,
    Removing,
    Paused,
    Exited,
    Dead,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameServerInfo {
    id: String,
    name: String,
    state: ContainerState,
    internal_host: String,
    internal_port: u16,
    external_port: u16,
}

impl From<Container> for GameServerInfo {
    fn from(value: Container) -> Self {
        let address = get_address(&value.ports);
        GameServerInfo {
            id: value.id,
            name: value.names,
            state: value.state,
            internal_host: address.internal_host,
            internal_port: address.internal_port,
            external_port: address.external_port,
        }
    }
}

struct Address {
    internal_host: String,
    internal_port: u16,
    external_port: u16,
}

/// The Docker container `ports` field is formatted `0.0.0.0:1xxxx->9000/tcp`.
/// xxxx is forwarded to 1xxxx for TLS via NGINX, then 1xxxx is forwarded to 9000 in Docker's internal network.
fn get_address(ports: &str) -> Address {
    let (internal_host, ports) = ports
        .split_once(":")
        .expect("Docker container `ports` has incorrect format");

    let (internal_port, external_port) = ports
        .split_once("->")
        .expect("Docker container `ports` has incorrect format");

    let internal_port = internal_port
        .parse::<u16>()
        .expect("Failed to parse internal port");

    let (external_port, _) = external_port
        .split_once("/")
        .expect("Docker container `ports` has incorrect format");

    let external_port = external_port
        .parse::<u16>()
        .expect("Failed to parse external port");

    Address {
        internal_host: internal_host.to_owned(),
        internal_port,
        external_port,
    }
}

#[get("/list/")]
async fn list(service_key: ServiceKey) -> actix_web::Result<HttpResponse> {
    service_key.validate()?;

    let output = Command::new("docker")
        .arg("ps")
        .arg("--format=json")
        .arg(format!(
            "--filter=ancestor={}",
            config::GAME_SERVER_IMAGE_NAME.to_owned(),
        ))
        .output()
        .map_err(|_| error::ErrorInternalServerError("Failed to list game servers"))?;

    if !output.status.success() {
        return Err(error::ErrorInternalServerError(
            "Failed to list game servers",
        ));
    }

    let services: Vec<GameServerInfo> = output
        .stdout
        .split(|&b| b == b'\n')
        .filter(|bytes| bytes.len() > 0)
        .filter_map(|bytes| String::from_utf8(bytes.into()).ok())
        .filter_map(|line| serde_json::from_str::<Container>(&line).ok())
        .filter_map(|container| {
            println!("{:?}", container);
            Some(GameServerInfo::from(container))
        })
        .collect();

    Ok(HttpResponse::Ok().json(services))
}
