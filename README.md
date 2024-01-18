# Server Authoritative Multiplayer

A sample project with matchmaking and server authority.

# Network Architecture

## WebRTC

We use WebRTC for its ability to provide low-latency, real-time, and secure communication between peers across all platforms. Most notably, WebRTC is the only such protocol that is available on the web.

This describes the implementation details of how the game server and game clients coordinate the creation and initialization of the WebRTC mesh network.

## Overview

### Authentication

When a player lauches a client, it establishes and maintains a connection to a matchmaking server. It authenticates with the matchmaking server to retrieve user profile information and matchmaking details. This unique user ID will be used to identify the user across multiple connections.

Once authenticated, the player can configure their party, manage their account, or indicate interest in starting a game. These actions and parameters are sent to the matchmaking server along with the game version for versioning.

### Starting a match

When the matchmaking server successfully configures a match, it will launch a game server instance at a pre-determined address. The matchmaking server will then send all matched players the game server address and match parameters.

When the client receives the game server information, it will initiate an RTC connection to the game server (most likely with WebRTC). The game server will coordinate the RTC mesh network setup.

### Leaving a match

When the client terminates its connection to the game server, either intentionally or via a dropped connection, the game server will drop the client and indicate this information to the matchmaking server.

### Joining an existing match

When a client reconnects to the matchmaking server, the matchmaking server will look up its database of player information to check their current status. If the player is within a validity window, the matchmaking server will send a message to the client indicating that re-joining the match is allowed with the game server information.

The client can respond to this message by ignoring or accepting the re-join offer and re-initiate an RTC connection to the game server.

# Server Authority Architecture

## Predictive events

Certain actions will trigger events that require immediate feedback e.g. hit registration for weapons. These events must be verified on the server but also need to be precomputed on the client. If verification fails, we need to either undo and rollback state or ignore the event's effects on the client.

We'll refer to these as **predictive events**. Predictive events should be delivered reliably.

### Predictive event stages

On the originating client, a predictive event has the following stages:

1. emit the event to the server
2. receive a verification from server
3. receive a rejection from server
4. timeout on the event

On the server, a predictive event has the following stages:

1. receive an event
2. either emit a verification or rejection to the originating client
3. if verified, broadcast the event to other clients.

On other clients, a predictive event will simply be received from the server pre-verified.

## Implementation

The client performs an RPC to the server to indicate the emission of a predictive event. Since the game state is replicated on the server and clients, we can co-locate our client-side emission code with our server-side handling code.

```gdscript
@rpc("reliable", "any_peer") # called by clients to the server.
func _spawn_player(event_id: int, at_position: Vector3) -> void:
  # handle server-side verification.


signal _spawn_player_response_signal(event_id: int, response: Variant)
@rpc("reliable") # called by the server for a given client.
func _spawn_player_response(event_id: int, response: Variant) -> void:
  _spawn_player_response_signal.emit(event_id, response)
```

To uniquely associate the event to the response from the server, we add an `event_id` argument and wrap the behavior within a `Promise` using our `Network.server_rpc` method.

```gdscript
func spawn_player(at_position: Vector3) -> void:
  var result = Events.server_rpc(_spawn_player, at_position, _spawn_player_response_signal).settled
```

# Variables

## Program

The universal interface for both servers and clients.

| Variable  | Description                        | Default value |
| --------- | ---------------------------------- | ------------- |
| `VERSION` | The semver version of the program. | `0.0.1`       |

## Server

| Variable         | Description                                                | Default value |
| ---------------- | ---------------------------------------------------------- | ------------- |
| `SERVER_ID`      | The unique ID to identify the server in logs.              | `randi()`     |
| `PORT`           | The port that the server listens for clients on.           | `8910`        |
| `SERVER_TIMEOUT` | How long the server will wait for a response from clients. | `15`          |

## Client

| Variable         | Description                                                   | Default value |
| ---------------- | ------------------------------------------------------------- | ------------- |
| `CLIENT_TIMEOUT` | How long the client will wait for a response from the server. | `15`          |
