extends GameServer
class_name WebSocketGameServer

var peer := WebSocketMultiplayerPeer.new()

@export var world_spawner: MultiplayerSpawner


func _ready() -> void:
	var start_result := start()

	if start_result.is_err():
		Logger.server_log(["failed to start server due to: ", start_result.unwrap_err()], ["init"])
		# For a dedicated server, if we fail to start the server, we want to
		# exit the program with an error code so that the matchmaking server
		# can detect the failure.
		if Program.is_dedicated_server:
			OS.kill(OS.get_process_id())
		# For prototyping, `_ready` is called by all clients but only one client
		# will be able to bind to the listening port. Therefore, we can free all
		# programs that failed to start the server and assume they are just clients.
		elif OS.is_debug_build():
			queue_free()
		return
	
	Logger.server_log(["started server on port: ", port], ["init"])
	# Get rid of this program's client.
	Program.client.queue_free()
	await Program.client.tree_exited

	multiplayer.multiplayer_peer = peer
	multiplayer.peer_connected.connect(_handle_peer_connected)
	multiplayer.peer_disconnected.connect(_handle_peer_disconnected)

	load_world()


func start() -> Result:
	Logger.server_log(["starting server on: ", port], ["init"])
	var result := Result.from_gderr(peer.create_server(port))
	if result.is_err():
		Logger.server_log(["failed to start server due to: ", result.unwrap_err()], ["init"])
	else:
		Logger.server_log(["started server on port: ", port], ["init"])
	return result


func _handle_peer_connected(peer_id: int) -> void:
	Logger.server_log(["client connected: ", peer_id], ["network"])
	var spawn_result := Program.world.spawn_player({
		"player_id": peer_id,
	})
	print(spawn_result)
	if spawn_result.is_err():
		Logger.server_log(["failed to spawn player(", peer_id, "): ", spawn_result.unwrap_err()], ["game"])


func _handle_peer_disconnected(peer_id: int) -> void:
	Logger.server_log(["client disconnected: ", peer_id], ["network"])
	var unspawn_result := Program.world.unspawn_player(peer_id)
	if unspawn_result.is_err():
		Logger.server_log(["failed to unspawn player(", peer_id, "): ", unspawn_result.unwrap_err()], ["game"])


#region Game Logic
const DEFAULT_WORLD_SCENE := "res://world/game_world.tscn"
func load_world(world_scene := DEFAULT_WORLD_SCENE) -> Result:
	var game_world = load(world_scene).instantiate()
	if not game_world is GameWorld:
		return Result.Err("Node(" + world_scene + ") is not a `GameWorld` instance")
	world_spawner.add_child(game_world)
	return Result.Ok(null)
#endregion
