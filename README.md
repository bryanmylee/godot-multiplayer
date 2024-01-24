# Server Authoritative Multiplayer

A sample project with matchmaking and server authority.

# Network Architecture

## Central Server

We use a central Linux server that serves as the first contact for game clients. The central server coordinates game clients to services like matchmaking and game servers.

## Game Server

Every game is run in a Docker container on a game server. A single game server can therefore host multiple games by running multiple containers bound to different ports selected by the matchmaking server.

We also run a simple REST API for spawning and stopping containers for game instances.

An NGINX proxy provides TLS by forwarding ports defined below:

| Internal service port | External port with TLS | Protocol  | Description                 |
| --------------------- | ---------------------- | --------- | --------------------------- |
| `80`                  | `443`                  | HTTP      | The container-spawning API. |
| `19000-19249`         | `9000-9249`            | WebSocket | The game server.            |

## Variables

### Program

The universal interface for both servers and clients.

| Variable  | Description                        | Default value |
| --------- | ---------------------------------- | ------------- |
| `VERSION` | The semver version of the program. | `0.0.1`       |

### Server

| Variable         | Description                                                | Default value |
| ---------------- | ---------------------------------------------------------- | ------------- |
| `SERVER_ID`      | The unique ID to identify the server in logs.              | `randi()`     |
| `PORT`           | The port that the server listens for clients on.           | `8910`        |
| `SERVER_TIMEOUT` | How long the server will wait for a response from clients. | `5.0`         |

### Client

| Variable         | Description                                                   | Default value |
| ---------------- | ------------------------------------------------------------- | ------------- |
| `SERVER_HOST`    | The server host to connect to.                                | `"127.0.0.1"` |
| `SERVER_PORT`    | The port on the server to connect to.                         | `8910`        |
| `CLIENT_TIMEOUT` | How long the client will wait for a response from the server. | `5.0`         |

# Roadmap

- [x] WebRTC Multiplayer Setup
- [x] Server Authoritative Synchronization
- [x] WebSocket Multiplayer Setup
- [ ] Game Server Deployment
- [ ] Authentication
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
