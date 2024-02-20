extends Control
class_name DebugAuthScreen

var logged_in := false :
	set(new):
		logged_in = new
		if logged_in:
			$NotLoggedIn.hide()
			$LoggedIn.show()
			$LoggedIn/UserName.text = Authentication.user_name.unwrap_or("")
			$LoggedIn/UserId.text = Authentication.user_id.unwrap_or("")
		else:
			$NotLoggedIn.show()
			$LoggedIn.hide()


func _ready() -> void:
	_on_sign_in_button_pressed()


func _on_sign_in_button_pressed() -> void:
	var auth_result := await Authentication.sign_in()
	print(auth_result)
	logged_in = Authentication.user_id.is_some()


@onready var game_server_address_input := $LoggedIn/ManualJoin/GameServerAddressInput as LineEdit
func _on_join_server_button_pressed():
	Program.main.load_game_screen({
		"start_client": true,
		"game_server_address": game_server_address_input.text,
	})


func _on_join_queue_button_pressed():
	pass # Replace with function body.


func _on_leave_queue_button_pressed():
	pass # Replace with function body.

@onready var server_port_input := $LoggedIn/Debug/DebugServer/ServerPortInput as LineEdit
func _on_start_debug_server_button_pressed():
	Program.main.load_game_screen({
		"start_server": true,
		"server_port": server_port_input.text,
	})


func _on_start_local_game_pressed():
	pass # Replace with function body.
