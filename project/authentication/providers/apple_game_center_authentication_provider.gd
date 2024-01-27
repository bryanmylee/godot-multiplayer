extends AuthenticationProvider
class_name AppleGameCenterAuthenticationProvider


func _init() -> void:
	name = "AppleGameCenter"


var game_center


func initialize() -> Result:
	if not Engine.has_singleton("GameCenter"):
		set_process(false)
		return Result.Err("GameCenter not available")
	
	game_center = Engine.get_singleton("GameCenter")
	game_center.authenticate()

	var authentication_result = await pending_event

	if authentication_result.result != "ok":
		return Result.Err("failed to authenticate with Game Center")
	
	user_id = authentication_result.player_id
	user_name = authentication_result.displayName
	print("Logged in with Game Center as ", user_name)

	return Result.Ok(null)


signal pending_event(payload: Variant)
func _process(_delta: float) -> void:
	if game_center.get_pending_event_count() > 0:
		var event = game_center.pop_pending_event()
		print("Game Center received event: ", event)
		pending_event.emit(event)
