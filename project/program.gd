extends Node


func _get_cmdline_args() -> Dictionary:
	var args := {}
	for arg in OS.get_cmdline_args():
		if arg.find("=") > -1:
			var key_value = arg.lstrip("-").split("=")
			args[key_value[0]] = key_value[1]
		else:
			args[arg.lstrip("-")] = true
	return args

var cmdline_args := _get_cmdline_args()
var is_dedicated_server := "server" in cmdline_args

var main: Main
var game_server: GameServer
var game_client: GameClient
var game_world: GameWorld
