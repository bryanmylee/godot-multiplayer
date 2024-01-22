# Server Authoritative WebRTC Multiplayer

A sample project with matchmaking, WebRTC, and server authority.

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
