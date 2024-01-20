extends Node

var is_dedicated_server := "--server" in OS.get_cmdline_args()
var ssl_enabled := not OS.is_debug_build()
var version := OS.get_environment("VERSION")

var server: GameServer
var client: GameClient
var world: GameWorld

var game_authority_id := 1
var is_game_authority:
	get: return server != null