extends Node3D
class_name GameWorld


@export var player_spawner: MultiplayerSpawner


func _enter_tree() -> void:
	Program.game_world = self
	set_multiplayer_authority(1)


func _ready() -> void:
	player_spawner.spawn_function = _handle_spawn_player


## [codeblock]
## @param opts SpawnPlayerOptions
## [/codeblock]
func _handle_spawn_player(opts) -> Node:
	var scene_path = opts.get("scene_path", DEFAULT_PLAYER_SCENE)
	var player = load(scene_path).instantiate()
	if not player is Player:
		print("Node(" + scene_path + ") is not a `Player` instance")
		return null

	player.set_multiplayer_authority(opts.player_id)
	player.name = str(opts.player_id)
	var spawn_position = opts.get("position", Vector3(randf_range(-10, 10), 0, randf_range(-10, 10)))
	player.position = spawn_position
	return player


const DEFAULT_PLAYER_SCENE := "res://game/player/player.tscn"
## [codeblock]
## @param opts SpawnPlayerOptions {
##   player_id: int
##   scene_path?: String
##   position?: Vector3
## }
##
## @returns Result<Player, String>
## [/codeblock]
func spawn_player(opts: Dictionary) -> Result:
	if not multiplayer.is_server():
		return Result.Err("authority-only method")
	Logger.server_log(["spawning player: ", opts.player_id])
	var spawned := player_spawner.spawn(opts)
	if spawned == null:
		return Result.Err("failed to spawn player %d" % [opts.player_id])
	return Result.Ok(spawned)


## [codeblock]
## @returns Result<Player, String>
## [/codeblock]
func unspawn_player(player_id: int) -> Result:
	if not multiplayer.is_server():
		return Result.Err("authority-only method")
	
	var player_result := Option.new(
		player_spawner.find_child(str(player_id), false)
	).ok_or("cannot find Player(" + str(player_id) + ") to unspawn")
	
	if player_result.is_err():
		return player_result

	player_result.unwrap().queue_free()
	return Result.Ok(null)
