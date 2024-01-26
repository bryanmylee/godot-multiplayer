extends AuthenticationProvider
class_name SteamAuthenticationProvider


func _init() -> void:
	name = "Steam"


func initialize() -> Result:
	OS.set_environment("SteamAppId", str(480))
	OS.set_environment("SteamGameId", str(480))

	Steam.steamInit()
	if not Steam.isSteamRunning():
		return Result.Err("Steam's API could not be initialized.\nEnsure that Steam is running.")
	print("Steam API initialized")

	user_id = str(Steam.getSteamID())
	user_name = Steam.getPersonaName()
	print("Logged in with Steam as ", user_name)

	return Result.Ok(null)