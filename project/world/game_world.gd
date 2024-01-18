extends Node3D
class_name GameWorld


@export var player_spawner: MultiplayerSpawner


func _enter_tree() -> void:
	Program.world = self
	set_multiplayer_authority(Program.get_multiplayer_authority())


func _ready():
	if Program.server == null:
		spawn_player()


#region spawn_player
"""
type SpawnPlayerOptions = {
	scene_path?: String;
	position?: Vector3;
}
"""
const DEFAULT_PLAYER_SCENE := "res://player/player.tscn"
func spawn_player(opts: Dictionary = {}) -> void:
	print("client(", multiplayer.get_unique_id(), "): spawning player")
	var spawn_result: Result = await GameNetwork.server_rpc(
		_spawn_player_server,
		opts,
		_spawn_player_settled,
	).settled
	print(spawn_result)

@rpc("reliable", "any_peer")
func _spawn_player_server(event_id: int, opts: Dictionary) -> void:
	"""
	@param opts: SpawnPlayerOptions
	"""
	var sender_id := multiplayer.get_remote_sender_id()
	print("server(", Program.server.id, "): spawning player: ", sender_id)

	var scene_path = opts.scene_path if "scene_path" in opts else DEFAULT_PLAYER_SCENE
	var player = load(scene_path).instantiate()
	if not player is Player:
		_spawn_player_response.rpc_id(sender_id, event_id, Result.Err(
			"Node (" + scene_path + ") is not a `Player` instance"
		).to_dict())
		return
	player.name = str(sender_id)
	
	var spawn_position = opts.position \
		if "position" in opts \
		else Vector3(randf_range(-10, 10), 0, randf_range(-10, 10))
	player.position = spawn_position
	
	player_spawner.add_child(player, true)
	_spawn_player_response.rpc_id(sender_id, event_id, Result.Ok(null).to_dict())

signal _spawn_player_settled(event_id: int, response: Variant)
@rpc("reliable")
func _spawn_player_response(event_id: int, response: Variant) -> void:
	_spawn_player_settled.emit(event_id, response)
#endregion
