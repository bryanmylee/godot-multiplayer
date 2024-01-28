extends Node
class_name Main

@onready var user_name_label := $UI/Container/UserName as Label
@onready var user_id_label := $UI/Container/UserId as Label


func _ready() -> void:
	var auth_result := await Authentication.initialize_default()
	if auth_result.is_err():
		print(auth_result.unwrap_err())
	else:
		user_name_label.text = Authentication.main_provider.user_name
		user_id_label.text = Authentication.main_provider.user_id
