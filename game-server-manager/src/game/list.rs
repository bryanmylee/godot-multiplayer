use actix_web::{error, get, HttpResponse};
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::config;

/**
{"Command":"\"./server --server -…\"",
"CreatedAt":"2024-02-17 03:30:35 +0800 +08",
"ID":"ff1ddbe07182",
"Image":"bryanmylee/multiplayer-base-game-server",
"Labels":"com.docker.compose.project.config_files=/Users/bryan/Projects/games/MultiplayerBase/compose.yaml,com.docker.compose.project.working_dir=/Users/bryan/Projects/games/MultiplayerBase,com.docker.compose.config-hash=b160d4e17130c0554daa76116f8183b80e12069abe36f4d729653e9c0f5d9c3e,com.docker.compose.image=sha256:5bfe01520e87a56dd7be62415574b85e0643816c980aec309466dc878b71e894,com.docker.compose.oneoff=False,com.docker.compose.project=multiplayerbase,com.docker.compose.service=game-server,com.docker.compose.version=2.23.0,com.docker.compose.container-number=1,com.docker.compose.depends_on=authentication:service_started:false"
"LocalVolumes":"0",
"Mounts":"",
"Names":"multiplayerbase-game-server-1",
"Networks":"multiplayerbase_default",
"Ports":"0.0.0.0:19000-\u003e9000/tcp",
"RunningFor":"14 hours ago",
"Size":"0B",
"State":"running",
"Status":"Up 14 hours"
}
 */

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
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
async fn list() -> actix_web::Result<HttpResponse> {
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
