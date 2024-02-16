extends Node

var is_dedicated_server := "--server" in OS.get_cmdline_args()
const SSL_ENABLED := false
const AUTH_SERVER_URI := "http://localhost:8000"
const AUTH_SERVER_PLAY_GAMES_OAUTH_CLIENT_ID := "865539732998-0ln5v17qagvfja9hlnb4rtf62ps21p2k.apps.googleusercontent.com"
const AUTH_SERVER_STEAM_IDENTITY = "authentication"
const IOS_BUNDLE_ID = "com.bryanmylee.multiplayer-base"
const STEAM_APP_ID = 2843770
const STEAM_GAME_ID = 2843770


var main: Main
var game_server: GameServer
var game_client: GameClient
var game_world: GameWorld
