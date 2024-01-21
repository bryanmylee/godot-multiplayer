extends Node
class_name Synchronized

@export_group("Dependencies")
@export var id_provider: IdentityProvider


func sync_to_authority(property: StringName, value: Variant):
	sync.rpc_id(Program.game_authority_id, property, value)


func authority_sync_to_peers(property: StringName, value: Variant):
	GameNetwork.rpc_clients_except_id(id_provider.id, sync, property, value)


func sync_reliable_to_authority(property: StringName, value: Variant):
	sync_reliable.rpc_id(Program.game_authority_id, property, value)


func authority_sync_reliable_to_peers(property: StringName, value: Variant):
	GameNetwork.rpc_clients_except_id(id_provider.id, sync_reliable, property, value)


@rpc
func sync(property: StringName, value: Variant):
	set(property, value)


@rpc("reliable")
func sync_reliable(property: StringName, value: Variant):
	set(property, value)
