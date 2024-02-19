# Server Authoritative Multiplayer

A minimal Godot project with cross-platform authentication, matchmaking, and server-authoritative multiplayer.

# Development Setup

Set up the Godot project as usual. Some export platforms require extra configuration.

## iOS and Game Center

We manage a custom fork of the official [`godot-ios-plugins`](https://github.com/bryanmylee/godot-ios-plugins) repo, built for Godot 4.2.

Plugins should be exported from [`godot-ios-plugins/`](./godot-ios-plugins/) and stored in `project/ios/`. Refer to the [iOS plugins document](./project/ios/README.md) for build instructions.

# Deployment

We currently manage our services with Docker Compose. Refer to [this guide](https://www.docker.com/blog/how-to-deploy-on-remote-docker-hosts-with-docker-compose/) on deploying the services to a remote host using Docker Contexts.

```bash
docker compose build
docker compose push
docker context create {context_name} --docker "host=ssh://{remote_user}@{remote_ip}"
```

The [`secrets/`](./secrets/) directory needs to be copied to all remote hosts with the exact same absolute location before deploying the project due to the [bind mount requirement](https://docs.docker.com/engine/swarm/services/#bind-mounts).

```bash
# On the remote host
mkdir -p {absolute_path_to_secrets}/secrets

# On the local host
scp -r secrets/ {remote_user}@{remote_ip}:{absolute_path_to_secrets}/
```

Then, deploy the services in the remote context.

```bash
docker --context {context_name} compose up --detach

# Check the processes
docker --context {context_name} ps
```

Lastly, refer to the [proxy setup document](nginx/README.md) to setup the proxy for the remote host.

# Network Architecture

## Proxy Server

We use an NGINX proxy that provides the main entrypoint to the multiplayer system and provides TLS.

Refer to the [proxy setup document](nginx/README.md).

## Authentication

Different platforms require different authentication strategies, including Steam, Apple Game Center, Google Play Games Services, and OAuth 2.0. All authentication strategies are consolidated and managed by our authentication server.

Refer to the [client-side authentication document](project/authentication/README.md) and the [authentication server document](authentication/README.md).

An NGINX proxy provides TLS by forwarding ports defined below:

| Internal service port | External proxy port | Protocol | Description             |
| --------------------- | ------------------- | -------- | ----------------------- |
| `18000`               | `8000`              | HTTP     | The authentication API. |

## Game Server Manager

A single host can host multiple game matches by running multiple Docker containers for each game match. The game server manager facilitates the spawning and killing of these containers, assigning an available port for each game.

Communication to these services is usually prohibited from outside the internal network, so a service key is sufficient.

| Internal service port | External proxy port | Protocol | Description              |
| --------------------- | ------------------- | -------- | ------------------------ |
| `18500`               | `8500`              | HTTP     | The game server manager. |

## Game Server

Every game match is run in a Docker container on a game server. A single game server can therefore host multiple matches by running multiple containers bound to different ports selected by the matchmaking server.

An NGINX proxy provides TLS by forwarding ports defined below:

| Internal service port | External proxy port | Protocol  | Description      |
| --------------------- | ------------------- | --------- | ---------------- |
| `19000-19249`         | `9000-9249`         | WebSocket | The game server. |

### Configuration

| Argument      | Description                                      | Default value |
| ------------- | ------------------------------------------------ | ------------- |
| `--server-id` | The unique ID to identify the server in logs.    | `randi()`     |
| `--port`      | The port that the server listens for clients on. | `9000`        |

## Game Client

The game has exports for Windows, macOS, Linux, iOS, Android, and the Web.

### Web Client

The web client is hosted on a simple webpage and acts like a local game client when its assets are downloaded.

An NGINX proxy provides TLS by forwarding ports defined below:

| Internal service port | External proxy port | Protocol | Description     |
| --------------------- | ------------------- | -------- | --------------- |
| `10443`               | `443`               | HTTP     | The web client. |

# Roadmap

- [x] WebRTC Multiplayer Setup
- [x] Server Authoritative Synchronization
- [x] WebSocket Multiplayer Setup
- [x] Game Server Deployment
- [x] Client-side Authentication
  - [x] Steam for Desktop
  - [x] Apple Game Center for iOS
  - [x] Google Play Games for Android
  - [x] OAuth 2.0 / OpenID for Web
- [ ] ~~Scalable Server-Authoritative Multiplayer~~
- [x] Server-side Authentication
  - [x] Authentication Server
  - [x] Steam for Desktop
  - [x] Apple Game Center for iOS
  - [x] Google Play Games for Android
  - [x] OAuth 2.0 / OpenID for Web
- [ ] Matchmaking

# Design considerations

## Why not use WebRTC for game state replication?

We initially did use WebRTC for its ability to provide low-latency, real-time, and secure communication between peers across all platforms. Most notably, WebRTC is the only two-way UDP-like protocol that is available on the web platform, since WebSockets rely on a TCP connection.

Our initial approach was to setup a WebRTC peer in the mesh network that was controlled by the game authority. However, deploying the server proved to be a challenge due to failing connectivity establishment for the server's WebRTC peer.

Furthermore, WebRTC required expensive TURN relay servers for the common case where connectivity cannot be directly established, either due to NAT forwarding or firewalls.

The scalability of WebRTC was also a challenge since bandwidth throughout the network scaled polynomially as the number of players increased. If we want to support multiplayer games with more than 4 players, WebRTC will struggle to maintain connectivity and bandwidth efficiency.

Lastly, the complexity of establishing the WebRTC network makes debugging and future maintenance difficult.

If a code example is needed, refer to the `webrtc-game-authoritative` tag in the commit history of this repo.

In the future, we could still leverage WebRTC for proximity voice chat or any other non-authoritative system using [`SceneTree::set_multiplayer`](https://docs.godotengine.org/en/stable/classes/class_scenetree.html#class-scenetree-method-set-multiplayer).
