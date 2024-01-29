extends Node

var is_dedicated_server := "--game_server" in OS.get_cmdline_args()
var ssl_enabled := not OS.is_debug_build()
var version := OS.get_environment("VERSION")

var main: Main

var game_server: GameServer
var game_client: GameClient
var game_world: GameWorld
