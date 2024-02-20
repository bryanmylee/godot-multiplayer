extends Node
class_name GameClient

## We use this in place of `multiplayer.get_unique_id()` for more customization and static access via `Program.game_client`.
var peer_id := 0
var peer := WebSocketMultiplayerPeer.new()

var game_server_address: String

func _init(_game_server_address: String) -> void:
	name = "GameClient"
	game_server_address = _game_server_address


## [codeblock]
## @returns Result<null, int>
## [/codeblock]
func start() -> Result:
	Logger.client_log(["starting client connection to game server at: ", game_server_address], ["init"])
	var start_result := Result.from_gderr(peer.create_client(game_server_address))

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
