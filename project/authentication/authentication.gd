extends Node
class_name Authentication

enum AuthenticationProvider {
	STEAM = 0,
	GOOGLE_PLAY_GAMES = 1,
	APPLE_GAME_CENTER = 2,
	OPEN_ID = 3,
}

var provider: AuthenticationProvider
var user_id: String
var user_name: String


func initialize() -> Result:
	var err := Result.Err("`initialize` not yet implemented")
	push_error(err.unwrap_err())
	return err
