extends AuthProvider
class_name AppleGameCenterAuthProvider

@onready var game_center: AppleGameCenterService = ServiceManager.get_service("AppleGameCenter")


func _init() -> void:
	name = "AppleGameCenter"


func initialize() -> Result:
	var auth_result: Result = await game_center.authenticate().settled
	if auth_result.is_err():
		return auth_result
	var auth_data = auth_result.unwrap()

	user_id = auth_data.player_id
	user_name = auth_data.displayName
	print("Logged in with Game Center as ", user_name)

	return Result.Ok(null)
