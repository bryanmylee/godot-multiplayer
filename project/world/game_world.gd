extends Node3D
class_name GameWorld


@export var player_spawner: MultiplayerSpawner


func _enter_tree() -> void:
	Program.world = self
	set_multiplayer_authority(Program.game_authority_id)


func _ready() -> void:
	if not Program.is_server:
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
	Logger.client_log(["spawning own player"], ["game", "world"])
	var spawn_result: Result = await GameNetwork.rpc_authority_with_return(
		__authority_spawn_player,
		opts,
		__settled_spawn_player,
	).settled
	Logger.client_log([spawn_result], ["game", "world"])


@rpc("reliable", "any_peer")
func __authority_spawn_player(event_id: int, opts: Dictionary) -> void:
	"""
	@param opts: SpawnPlayerOptions
	"""
	var sender_id := multiplayer.get_remote_sender_id()
	Logger.server_log(["spawning player: ", sender_id], ["game", "world"])

	var scene_path = opts.get("scene_path", DEFAULT_PLAYER_SCENE)
	var player = load(scene_path).instantiate()
	if not player is Player:
		_spawn_player__response.rpc_id(sender_id, event_id, Result.Err(
			"Node (" + scene_path + ") is not a `Player` instance"
		).to_dict())
		return
	player.name = str(sender_id)
	
	var spawn_position = opts.get("position", Vector3(randf_range(-10, 10), 0, randf_range(-10, 10)))
	player.position = spawn_position
	
	player_spawner.add_child(player, true)
	_spawn_player__response.rpc_id(sender_id, event_id, Result.Ok(null).to_dict())


signal __settled_spawn_player(event_id: int, response: Variant)
@rpc("reliable")
func _spawn_player__response(event_id: int, response: Variant) -> void:
	__settled_spawn_player.emit(event_id, response)
#endregion
