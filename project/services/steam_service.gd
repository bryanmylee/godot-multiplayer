extends Service
class_name SteamService


func _init() -> void:
	name = "Steam"


func initialize() -> Result:
	OS.set_environment("SteamAppId", str(480))
	OS.set_environment("SteamGameId", str(480))

	Steam.steamInit()
	if not Steam.isSteamRunning():
		return Result.Err("Steam's API could not be initialized.\nEnsure that Steam is running.")
	print("Steam API initialized")

	return Result.Ok(null)
