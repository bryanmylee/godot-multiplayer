extends Node
class_name Main

@onready var transition := $SceneTransition as SceneTransition


func _ready() -> void:
	Program.main = self

	if Program.is_dedicated_server:
		await load_game_screen()
		return
	
	var init_result := await Authentication.initialize_main_provider()
	if init_result.is_err():
		print(init_result.unwrap_err())
		return
	var sign_in_result := await Authentication.sign_in()
	if sign_in_result.is_err():
		print(sign_in_result.unwrap_err())

	await load_home_screen()


func load_home_screen() -> void:
	var home_screen := preload("res://screens/home.tscn").instantiate()
	await transition.fade_to(home_screen)


## [codeblock]
## @param params GameOptions {
##   start_server?: bool
##   server_port?: int
##   start_client?: bool
##   game_server_address?: String
## }
## [/codeblock]
func load_game_screen(params := {}) -> void:
	var game_screen := preload("res://game/game.tscn").instantiate() as Game
	if params.has("start_server"):
		game_screen.start_server = params.start_server
	if params.has("server_port"):
		game_screen.server_port = params.server_port
	if params.has("start_client"):
		game_screen.start_client = params.start_client
	if params.has("game_server_address"):
		game_screen.game_server_address = params.game_server_address
	await transition.fade_to(game_screen)
