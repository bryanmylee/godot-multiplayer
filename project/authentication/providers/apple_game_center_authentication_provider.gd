extends AuthenticationProvider
class_name AppleGameCenterAuthenticationProvider


func _init() -> void:
	name = "AppleGameCenter"


func initialize() -> Result:
	return Result.Err("Apple Game Center not yet implemented")
