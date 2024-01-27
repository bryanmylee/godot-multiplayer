extends Node
class_name Main

@onready var authentication := $Authentication as Authentication


func _ready() -> void:
	var auth_result := await authentication.initialize_default()
	if auth_result.is_err():
		print(auth_result.unwrap_err())
