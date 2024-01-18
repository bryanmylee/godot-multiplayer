extends CharacterBody3D
class_name Player


func _enter_tree() -> void:
	set_multiplayer_authority(Program.get_multiplayer_authority())


func _ready() -> void:
	$Camera.current = name.to_int() == Program.client.peer_id
