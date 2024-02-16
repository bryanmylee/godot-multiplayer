extends Node
class_name GameClient

# const SERVER_HOST := "multiplayer-test.bryanmylee.com"
const SERVER_HOST := "127.0.0.1"

const SERVER_PORT := 9000

## We use this in place of `multiplayer.get_unique_id()` for more customization and static access via `Program.game_client`.
var peer_id := 0
var peer := WebSocketMultiplayerPeer.new()


func _init() -> void:
	name = "GameClient"


## [codeblock]
## @returns Result<null, int>
## [/codeblock]
func start() -> Result:
	var protocol := "wss://" if Env.SSL_ENABLED else "ws://"
	var address := protocol + SERVER_HOST + ":" + str(SERVER_PORT)
	Logger.client_log(["starting client connection to game server at: ", address], ["init"])
	var start_result := Result.from_gderr(peer.create_client(address))

	multiplayer.multiplayer_peer = peer
	multiplayer.connected_to_server.connect(_handle_connected_to_server)
	multiplayer.connection_failed.connect(_handle_server_connection_failed)
	multiplayer.server_disconnected.connect(_handle_server_disconnected)

	return start_result


#region Network
func _handle_connected_to_server() -> void:
	peer_id = multiplayer.get_unique_id()
	Logger.client_log(["connected to server"], ["init"])


func _handle_server_connection_failed() -> void:
	Logger.client_log(["failed to connect to server"], ["init"])


func _handle_server_disconnected() -> void:
	peer_id = 0
	Logger.client_log(["server disconnected"], ["network"])
#endregion
