extends Service
class_name GooglePlayGamesService

var core: PlayGamesServicesCore


func _init() -> void:
	name = "GooglePlayGames"


func initialize() -> Result:
	core = load("res://addons/godot-play-games-services/play_games_services_core.gd").new()
	add_child(core)
	var success = core.initialize()
	if not success:
		return Result.Err("Play Games Services not available")
	return Result.Ok(null)


func request_server_side_access() -> Result:
	return Result.Ok(null)
