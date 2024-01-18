extends CharacterBody3D
class_name Player

@onready var id_provider := $IdentityProvider as IdentityProvider


func _enter_tree() -> void:
	set_multiplayer_authority(Program.game_authority_id)


func _ready() -> void:
	$Camera.current = id_provider.is_local_player
