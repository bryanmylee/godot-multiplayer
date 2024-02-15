extends Node

var is_dedicated_server := "--server" in OS.get_cmdline_args()
const SSL_ENABLED := false
const AUTH_SERVER_URI := "http://localhost:8000"
const IOS_BUNDLE_ID = "com.bryanmylee.multiplayer-base"


var main: Main

var game_server: GameServer
var game_client: GameClient
var game_world: GameWorld
