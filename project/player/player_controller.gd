extends Node
class_name PlayerController

@export_group("Dependencies")
@export var id_provider: IdentityProvider


func _enter_tree() -> void:
	set_multiplayer_authority(id_provider.id)


#region Synchronized State
var is_crouching := false
var is_running := false
var direction := Vector2.ZERO
var just_jumped := false
#endregion


func _ready() -> void:
	set_process(id_provider.is_local_player)
	if id_provider.is_local_player:
		NetworkTime.after_tick_loop.connect(_clear_events)
		NetworkTime.before_tick_loop.connect(_gather_input)


func _process(_delta: float) -> void:
	if Input.is_action_just_pressed("jump"):
		just_jumped = true


func _clear_events() -> void:
	just_jumped = false


func _gather_input() -> void:
	is_crouching = Input.is_action_pressed("mod_crouch")
	is_running = Input.is_action_pressed("mod_run")
	direction = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
