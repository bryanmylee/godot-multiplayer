extends CharacterBody3D
class_name Player


@onready var player_id := $PlayerId as PlayerId
@onready var controller := $Controller as PlayerController
@onready var camera := $PitchPivot/Camera
@onready var pitch_pivot := $PitchPivot as Node3D
@onready var yaw_pivot := self as Node3D


func _ready() -> void:
	camera.current = player_id.is_local_player


func _network_process(delta: float, _tick: int) -> void:
	orientation(delta)
	movement(delta)


func orientation(_delta: float) -> void:
	pitch_pivot.rotation.x = controller.pitch
	yaw_pivot.rotation.y = controller.yaw


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
	
	if controller.just_jumped:
		_force_update_slide_collision()
		if is_on_floor():
			velocity.y = 4.0

	velocity *= NetworkTime.physics_factor
	move_and_slide()
	velocity /= NetworkTime.physics_factor


func _force_update_slide_collision() -> void:
	var old_velocity = velocity
	velocity = Vector3.ZERO
	move_and_slide()
	velocity = old_velocity


func _on_player_id_platform_changed(platform: String) -> void:
	_set_body_material(platform)


@export_group("Platform Materials")
@export var desktop_material: Material
@export var android_material: Material
@export var ios_material: Material
@export var web_material: Material
@export var xr_material: Material


func _set_body_material(platform: String) -> void:
	var mesh := $Model/BodyMesh as MeshInstance3D
	var material: Material
	match platform:
		"Windows", "macOS", "Linux", "FreeBSD", "NetBSD", "OpenBSD", "BSD":
			material = desktop_material
		"Android":
			material = android_material
		"iOS":
			material = ios_material
		"Web":
			material = web_material
		_:
			material = desktop_material
	mesh.material_override = material
