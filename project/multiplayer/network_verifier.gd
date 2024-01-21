extends Node
class_name NetworkVerifier

@export var id_provider: IdentityProvider
@export var verification_ticks_per_second := 10

var delta_verification := 1.0 / verification_ticks_per_second


func set_verification_process(enabled: bool) -> void:
	set_physics_process(enabled)


var elapsed_time_in_tick := 0.0


func _physics_process(delta: float) -> void:
	elapsed_time_in_tick += delta
	if elapsed_time_in_tick >= delta_verification:
		_verification_process()
		elapsed_time_in_tick -= delta_verification


func _verification_process() -> void:
	pass


func verify_on_peers(property: StringName, value: Variant) -> void:
	sync.rpc(property, value)


func verify_reliable_on_peers(property: StringName, value: Variant) -> void:
	sync_reliable.rpc(property, value)


@rpc
func sync(property: StringName, value: Variant) -> void:
	set(property, value)


@rpc("reliable")
func sync_reliable(property: StringName, value: Variant) -> void:
	set(property, value)
