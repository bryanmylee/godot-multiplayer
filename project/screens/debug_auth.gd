extends Control
class_name DebugAuthScreen

@onready var user_name_label := $UserName as Label
@onready var user_id_label := $UserId as Label
@onready var start_game_button := $StartGameButton as Button


func _ready() -> void:
	if Authentication.main_provider == null:
		user_name_label.text = "Not authenticated"
	else:
		user_name_label.text = Authentication.main_provider.user_name
		user_id_label.text = Authentication.main_provider.user_id
	start_game_button.pressed.connect(Program.main.load_game_screen)
