extends Node
class_name Main

@onready var transition := $SceneTransition as SceneTransition


func _ready() -> void:
	Program.main = self

	if Program.is_dedicated_server:
		await load_game_screen()
		return
	
	var provider_result := await Authentication.initialize_main_provider()
	if provider_result.is_err():
		print(provider_result.unwrap_err())
		return
	var provider: AuthProvider = provider_result.unwrap()

	var sign_in_result := await provider.server_sign_in()
	if sign_in_result.is_err():
		print(sign_in_result.unwrap_err())
		return
	var sign_in_data = sign_in_result.unwrap()
	print("Signed in to server!\n", sign_in_data)

	await load_debug_auth_screen()


func load_debug_auth_screen() -> void:
	var debug_auth_screen := preload("res://screens/debug_auth.tscn").instantiate()
	await transition.fade_to(debug_auth_screen)


func load_game_screen() -> void:
	var game_screen := preload("res://game/game.tscn").instantiate()
	await transition.fade_to(game_screen)
