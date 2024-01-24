extends Node
class_name GameClient

const _DEFAULT_TIMEOUT := 5.0
var env_timeout := OS.get_environment("CLIENT_TIMEOUT")
var timeout := float(env_timeout) if env_timeout else _DEFAULT_TIMEOUT

const _DEFAULT_SERVER_HOST := "127.0.0.1"
var env_server_host := OS.get_environment("SERVER_HOST")
var server_host := env_server_host if env_server_host else _DEFAULT_SERVER_HOST

const _DEFAULT_SERVER_PORT := 9000
var env_server_port := OS.get_environment("SERVER_PORT")
var server_port := int(env_server_port) if env_server_port else _DEFAULT_SERVER_PORT

## We use this in place of `multiplayer.get_unique_id()` for more customization and static access via `Program.client`.
var peer_id := 0
var peer := WebSocketMultiplayerPeer.new()


func _enter_tree() -> void:
	Program.client = self


func _ready() -> void:
	start()

	multiplayer.multiplayer_peer = peer
	multiplayer.connected_to_server.connect(_handle_connected_to_server)
	multiplayer.connection_failed.connect(_handle_server_connection_failed)
	multiplayer.server_disconnected.connect(_handle_server_disconnected)


func start() -> Result:
	var protocol := "wss://" if Program.ssl_enabled else "ws://"
	var address := protocol + server_host + ":" + str(server_port)
	Logger.client_log(["connecting to game server at: ", address], ["init"])
	return Result.from_gderr(peer.create_client(address))


#region Network
func _handle_connected_to_server() -> void:
	peer_id = multiplayer.get_unique_id()
	Logger.client_log(["connected to server"], ["init"])
	GameNetwork.game_network_ready.emit()


func _handle_server_connection_failed() -> void:
	Logger.client_log(["failed to connect to server"], ["init"])


func _handle_server_disconnected() -> void:
	peer_id = 0
	Logger.client_log(["server disconnected"], ["network"])
#endregion
