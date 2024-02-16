extends Node

var is_dedicated_server := "--server" in OS.get_cmdline_args()

var main: Main
var game_server: GameServer
var game_client: GameClient
var game_world: GameWorld
