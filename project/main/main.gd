extends Node
class_name Main

@onready var transition := $SceneTransition as SceneTransition


func _ready() -> void:
	Program.main = self

	if Program.is_dedicated_server:
		await load_game_screen()
		return
	
	var auth_result := await Authentication.initialize_default()
	if auth_result.is_err():
		print(auth_result.unwrap_err())
	await load_debug_auth_screen()


func load_debug_auth_screen() -> void:
	var debug_auth_screen := preload("res://screens/debug_auth.tscn").instantiate()
	await transition.fade_to(debug_auth_screen)


func load_game_screen() -> void:
	var game_screen := preload("res://game/game.tscn").instantiate()
	await transition.fade_to(game_screen)
