extends Node
class_name Main


func _ready() -> void:
	var auth_result := await Authentication.initialize_default()
	if auth_result.is_err():
		print(auth_result.unwrap_err())
