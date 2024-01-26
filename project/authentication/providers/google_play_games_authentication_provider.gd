extends AuthenticationProvider
class_name GooglePlayGamesAuthenticationProvider


func _init() -> void:
	name = "GooglePlayGames"


func initialize() -> Result:
	return Result.Err("Google Play Games Services not yet implemented")
