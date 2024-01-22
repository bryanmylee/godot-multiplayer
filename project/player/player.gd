extends CharacterBody3D
class_name Player

func _enter_tree() -> void:
	set_multiplayer_authority(Program.game_authority_id)


@onready var id_provider := $IdentityProvider as IdentityProvider
@onready var controller := $Controller as PlayerController


func _ready() -> void:
	$Camera.current = id_provider.is_local_player


func _rollback_tick(delta: float, _tick: int, _is_fresh: bool) -> void:
	movement(delta)


func movement(delta: float) -> void:
	var input_dir := Vector3(controller.direction.x, 0, controller.direction.y)
	var global_input_dir := transform.basis * input_dir

	if global_input_dir.is_zero_approx():
		var target_velocity := Vector3(0, velocity.y, 0)
		velocity = velocity.move_toward(target_velocity, 25 * delta)
	else:
		var target_speed := 8.0 if controller.is_running else 4.0
		var target_velocity := Vector3(
			global_input_dir.x * target_speed,
			velocity.y,
			global_input_dir.z * target_speed
		)
		velocity = velocity.move_toward(target_velocity, 30 * delta)
	
	velocity.y -= 9.8 * delta
	
	if controller.is_jumping:
		_force_update_is_on_floor()
		if is_on_floor():
			velocity.y = 4.0

	velocity *= NetworkTime.physics_factor
	move_and_slide()
	velocity /= NetworkTime.physics_factor


func _force_update_is_on_floor():
	var old_velocity = velocity
	velocity = Vector3.ZERO
	move_and_slide()
	velocity = old_velocity
