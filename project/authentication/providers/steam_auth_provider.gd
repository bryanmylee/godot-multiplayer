extends AuthProvider
class_name SteamAuthProvider


func _init() -> void:
	name = "Steam"


func initialize() -> Result:
	provider_type = "steam"
	provider_id = str(Steam.getSteamID())
	user_name = Option.Some(Steam.getPersonaName())
	print("Logged in with Steam as ", user_name.unwrap())

	return Result.Ok(null)
