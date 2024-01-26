extends AuthenticationProvider
class_name OpenIDAuthenticationProvider


func _init() -> void:
	name = "OpenID"


func initialize() -> Result:
	return Result.Err("OpenID not yet implemented")
