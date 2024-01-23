extends GameServer
class_name WebSocketGameServer

var peer := WebSocketMultiplayerPeer.new()


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


func _handle_peer_disconnected(peer_id: int) -> void:
	Logger.server_log(["client disconnected: ", peer_id], ["network"])
