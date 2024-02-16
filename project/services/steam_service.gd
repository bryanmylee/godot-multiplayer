extends Service
class_name SteamService


func _init() -> void:
	name = "Steam"


func initialize() -> Result:
	OS.set_environment("SteamAppId", str(Env.STEAM_APP_ID))
	OS.set_environment("SteamGameId", str(Env.STEAM_GAME_ID))

	Steam.steamInit()
	if not Steam.isSteamRunning():
		return Result.Err("Steam's API could not be initialized.\nEnsure that Steam is running.")
	print("Steam API initialized")

	return Result.Ok(null)


func _process(_delta: float) -> void:
	Steam.run_callbacks()
