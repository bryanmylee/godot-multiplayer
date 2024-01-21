extends Node3D
class_name GameWorld


@export var player_spawner: MultiplayerSpawner


func _enter_tree() -> void:
	Program.world = self
	set_multiplayer_authority(Program.game_authority_id)


"""
type SpawnPlayerOptions = {
	player_id: int;
	scene_path?: String;
	position?: Vector3;
}
"""
const DEFAULT_PLAYER_SCENE := "res://player/player.tscn"
func authority_spawn_player(opts: Dictionary) -> Result: # Result<Player>
	"""
	@param opts: SpawnPlayerOptions
	"""
	if not Program.is_game_authority:
		return Result.Err("authority-only method")
	
	var player_id = opts.player_id
	Logger.server_log(["spawning player: ", player_id], ["game", "world"])

	var scene_path = opts.get("scene_path", DEFAULT_PLAYER_SCENE)
	var player = load(scene_path).instantiate()
	if not player is Player:
		return Result.Err("Node(" + scene_path + ") is not a `Player` instance")
	
	player.name = str(player_id)
	
	var spawn_position = opts.get("position", Vector3(randf_range(-10, 10), 0, randf_range(-10, 10)))
	player.position = spawn_position
	
	player_spawner.add_child(player, true)
	player.owner = player_spawner
	return Result.Ok(player)


func authority_unspawn_player(player_id: int) -> Result: # Result<Player>
	if not Program.is_game_authority:
		return Result.Err("authority-only method")
	
	var player_result := Option.new(
		player_spawner.find_child(str(player_id), false)
	).ok_or("cannot find Player(" + str(player_id) + ") to unspawn")
	
	if player_result.is_err():
		return player_result

	player_result.unwrap().queue_free()
	return Result.Ok(null)
