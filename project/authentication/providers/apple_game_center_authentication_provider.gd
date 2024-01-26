extends AuthenticationProvider
class_name AppleGameCenterAuthenticationProvider


func _init() -> void:
	name = "AppleGameCenter"


var game_center


func initialize() -> Result:
	if not Engine.has_singleton("GameCenter"):
		return Result.Err("GameCenter not available")
	game_center = Engine.get_singleton("GameCenter")

	return Result.Err("Apple Game Center not yet implemented")
