extends Node
class_name PlayerInput

@export_group("Dependencies")
@export var id_provider: IdentityProvider


var is_crouching := false
var is_running := false
var direction := Vector2.ZERO

var jumped := false


func _ready():
	set_process(not id_provider.is_remote_player)
	set_process_input(not id_provider.is_remote_player)


func _process(_delta: float):
	# continuous input
	is_crouching = Input.is_action_pressed("mod_crouch")
	is_running = Input.is_action_pressed("mod_run")
	direction = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")

	# events
	jumped = Input.is_action_just_pressed("jump")
