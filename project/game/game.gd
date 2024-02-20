extends Node
class_name Game

@onready var world_spawner := $WorldSpawner as MultiplayerSpawner

@export var start_server := false
var server_port_str: String = Program.cmdline_args.get("port", "9000")
@export var server_port := int(server_port_str)

@export var start_client := false
@export var game_server_address := "ws://localhost:9000"


func _ready() -> void:
	# Production server build.
	if Program.is_dedicated_server:
		var server_result := _start_server()
		if server_result.is_err():
			OS.kill(OS.get_process_id())
		return
	
	if start_server:
		print(_start_server())
	
	if start_client:
		print(_start_client())


## [codeblock]
## @returns Result<null, int>
## [/codeblock]
func _start_server() -> Result:
	Program.game_server = GameServer.new(server_port, world_spawner)
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
	Program.game_client = GameClient.new(game_server_address)
	add_child(Program.game_client, true)

	var start_result := Program.game_client.start()
	if start_result.is_err():
		Program.game_client = null
		return start_result
	Logger.client_log(["started client"], ["init"])

	return start_result
