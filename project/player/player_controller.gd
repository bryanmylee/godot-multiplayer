extends Node
class_name PlayerController

@export_group("Dependencies")
@export var id_provider: IdentityProvider


func _enter_tree() -> void:
	set_multiplayer_authority(id_provider.id)


#region Synchronized State
var is_crouching := false
var is_running := false
var is_jumping := false
var direction := Vector2.ZERO
#endregion


func _ready() -> void:
	if id_provider.is_local_player:
		NetworkTime.before_tick_loop.connect(_gather_input)


func _gather_input() -> void:
	is_crouching = Input.is_action_pressed("mod_crouch")
	is_running = Input.is_action_pressed("mod_run")
	is_jumping = Input.is_action_pressed("jump")
	direction = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")