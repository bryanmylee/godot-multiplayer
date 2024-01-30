extends Node
class_name GameServer

const _DEFAULT_PORT := 9000
var env_port := OS.get_environment("PORT")
var port := int(env_port) if env_port else _DEFAULT_PORT

var env_id := OS.get_environment("SERVER_ID")
var id := int(env_id) if env_id else randi()

var peer := WebSocketMultiplayerPeer.new()
var world_spawner: MultiplayerSpawner


func _init(_world_spawner: MultiplayerSpawner) -> void:
	name = "GameServer"
	world_spawner = _world_spawner


func start() -> Result:
	Logger.server_log(["starting server on: ", port], ["init"])
	var start_result := Result.from_gderr(peer.create_server(port))
	if start_result.is_err():
		return start_result

	multiplayer.multiplayer_peer = peer
	multiplayer.peer_connected.connect(_handle_peer_connected)
	NetworkTime.after_client_sync.connect(_handle_peer_time_synced)
	multiplayer.peer_disconnected.connect(_handle_peer_disconnected)

	return start_result


#region Network
func _handle_peer_connected(peer_id: int) -> void:
	Logger.server_log(["client connected: ", peer_id], ["network"])


func _handle_peer_time_synced(peer_id: int) -> void:
	var spawn_result := Program.game_world.spawn_player({
		"player_id": peer_id,
	})
	if spawn_result.is_err():
		Logger.server_log(["failed to spawn player(", peer_id, "): ", spawn_result.unwrap_err()], ["game"])
	else:
		Logger.server_log(["spawned player(", peer_id, ")"], ["game"])


func _handle_peer_disconnected(peer_id: int) -> void:
	Logger.server_log(["client disconnected: ", peer_id], ["network"])
	var unspawn_result := Program.game_world.unspawn_player(peer_id)
	if unspawn_result.is_err():
		Logger.server_log(["failed to unspawn player(", peer_id, "): ", unspawn_result.unwrap_err()], ["game"])
#endregion


const DEFAULT_WORLD_SCENE := "res://game/world/game_world.tscn"
func load_world(world_scene := DEFAULT_WORLD_SCENE) -> Result:
	var game_world = load(world_scene).instantiate()
	if not game_world is GameWorld:
		return Result.Err("Node(" + world_scene + ") is not a `GameWorld` instance")
	world_spawner.add_child(game_world)
	return Result.Ok(null)
