extends Node
class_name PlayerController

@export_group("Dependencies")
@export var id_provider: IdentityProvider


@export_group("Settings")
@export var max_pitch_angle := deg_to_rad(89)
@export var min_pitch_angle := deg_to_rad(-89)
@export var mouse_sensitivity_y := 0.0008
@export var mouse_sensitivity_x := 0.0008


func _enter_tree() -> void:
	set_multiplayer_authority(id_provider.id)


#region Synchronized State
var is_crouching := false
var is_running := false
var direction := Vector2.ZERO
var just_jumped := false
var delta_pitch := 0.0
var delta_yaw := 0.0
#endregion


var _curr_tick_delta_pitch := 0.0
var _curr_tick_delta_yaw := 0.0


func _ready() -> void:
	set_process(id_provider.is_local_player)
	set_process_input(id_provider.is_local_player)
	if id_provider.is_local_player:
		NetworkTime.before_tick_loop.connect(_gather_input)
		NetworkTime.after_tick_loop.connect(_clear_input)


func _process(_delta: float) -> void:
	if Input.is_action_just_pressed("jump"):
		just_jumped = true


func _input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		_handle_mouse_motion(event)
	elif event is InputEventMouseButton:
		_handle_mouse_button(event)
	elif Input.is_action_pressed("ui_cancel"):
		escape_mouse_capture()


func _handle_mouse_motion(event: InputEventMouseMotion) -> void:
	if Input.mouse_mode != Input.MOUSE_MODE_CAPTURED:
		return
	_curr_tick_delta_pitch += event.relative.y * mouse_sensitivity_y
	_curr_tick_delta_yaw += event.relative.x * mouse_sensitivity_x


func _handle_mouse_button(_event: InputEventMouseButton) -> void:
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED


func escape_mouse_capture() -> void:
	Input.mouse_mode = Input.MOUSE_MODE_VISIBLE


func _gather_input() -> void:
	is_crouching = Input.is_action_pressed("mod_crouch")
	is_running = Input.is_action_pressed("mod_run")
	direction = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
	delta_pitch = _curr_tick_delta_pitch
	delta_yaw = _curr_tick_delta_yaw


func _clear_input() -> void:
	just_jumped = false
	_curr_tick_delta_pitch = 0.0
	_curr_tick_delta_yaw = 0.0
