extends Node
class_name Game

@onready var world_spawner := $WorldSpawner as MultiplayerSpawner


func _ready() -> void:
	# Production server build.
	if Program.is_dedicated_server:
		var server_result := _start_server()
		if server_result.is_err():
			OS.kill(OS.get_process_id())
		return
	
	# Production client build.
	if not OS.is_debug_build():
		var client_result := _start_client()
		print(client_result)
		return
	
	#_try_debug_server_or_client()
	_start_client()


func _try_debug_server_or_client() -> void:
	# In debug builds, we try loading the server on all programs since only one
	# program will be able to bind to the server port. This leaves us with just
	# one server.
	var server_result := _start_server()
	print(server_result)
	if server_result.is_err():
		var client_result := _start_client()
		print(client_result)


## [codeblock]
## @returns Result<null, int>
## [/codeblock]
func _start_server() -> Result:
	Program.game_server = GameServer.new(world_spawner)
	add_child(Program.game_server, true)

	var start_result := Program.game_server.start()
	if start_result.is_err():
		Program.game_server = null
		return start_result
	Logger.server_log(["started server"], ["init"])

	return Program.game_server.load_world()


## [codeblock]
## @returns Result<null, int>
## [/codeblock]
func _start_client() -> Result:
	Program.game_client = GameClient.new()
	add_child(Program.game_client, true)

	var start_result := Program.game_client.start()
	if start_result.is_err():
		Program.game_client = null
		return start_result
	Logger.client_log(["started client"], ["init"])

	return start_result
