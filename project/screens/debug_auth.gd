extends Control
class_name DebugAuthScreen

@onready var user_name_label := $UserName as Label
@onready var provider_id_label := $ProviderId as Label
@onready var server_user_id_label := $ServerUserId as Label


func _ready() -> void:
	if Authentication.main_provider == null:
		user_name_label.text = "Not authenticated"
		provider_id_label.text = ""
		server_user_id_label.text = ""
	else:
		user_name_label.text = Authentication.main_provider.user_name.unwrap_or("")
		provider_id_label.text = Authentication.main_provider.provider_id
		server_user_id_label.text = Authentication.user_id.unwrap_or("")


func _on_start_game_button_pressed():
	Program.main.load_game_screen()
