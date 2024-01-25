extends Node
class_name Main

@onready var authentication := $Authentication as Authentication


func _ready() -> void:
	var auth_result := initialize_authentication()
	if auth_result.is_err():
		print(auth_result.unwrap_err())


func initialize_authentication() -> Result:
	match OS.get_name():
		"Windows", "macOS", "Linux", "FreeBSD", "NetBSD", "OpenBSD", "BSD":
			authentication.set_script(load("res://authentication/steam_authentication.gd"))
		"Android":
			authentication.set_script(load("res://authentication/google_play_games_authentication.gd"))
		"iOS":
			authentication.set_script(load("res://authentication/apple_game_center_authentication.gd"))
		"Web":
			authentication.set_script(load("res://authentication/open_id_authentication.gd"))
	return authentication.initialize()
