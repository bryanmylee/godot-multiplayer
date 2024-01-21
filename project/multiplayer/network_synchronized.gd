extends Node
class_name NetworkSynchronized

@export var id_provider: IdentityProvider
@export var synchronization_ticks_per_second := 10

var delta_synchronization := 1.0 / synchronization_ticks_per_second


func set_synchronization_process(enabled: bool) -> void:
	set_physics_process(enabled)


var elapsed_time_in_tick := 0.0


func _physics_process(delta: float) -> void:
	elapsed_time_in_tick += delta
	if elapsed_time_in_tick >= delta_synchronization:
		_synchronization_process()
		elapsed_time_in_tick -= delta_synchronization


func _synchronization_process() -> void:
	pass


func sync_to_authority(property: StringName, value: Variant) -> void:
	sync.rpc_id(Program.game_authority_id, property, value)


func sync_reliable_to_authority(property: StringName, value: Variant) -> void:
	sync_reliable.rpc_id(Program.game_authority_id, property, value)


func authority_sync_to_peers(property: StringName, value: Variant) -> void:
	GameNetwork.rpc_clients_except_id(id_provider.id, sync, property, value)


func authority_sync_reliable_to_peers(property: StringName, value: Variant) -> void:
	GameNetwork.rpc_clients_except_id(id_provider.id, sync_reliable, property, value)


@rpc
func sync(property: StringName, value: Variant) -> void:
	set(property, value)


@rpc("reliable")
func sync_reliable(property: StringName, value: Variant) -> void:
	set(property, value)
