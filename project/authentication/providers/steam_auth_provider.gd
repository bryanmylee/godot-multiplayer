extends AuthProvider
class_name SteamAuthProvider


func _init() -> void:
	name = "Steam"


func initialize() -> Result:
	user_id = str(Steam.getSteamID())
	user_name = Steam.getPersonaName()
	print("Logged in with Steam as ", user_name)

	return Result.Ok(null)
