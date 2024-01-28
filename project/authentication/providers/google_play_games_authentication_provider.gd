extends AuthenticationProvider
class_name GooglePlayGamesAuthenticationProvider


func _init() -> void:
	name = "GooglePlayGames"


var player: PlayGamesPlayersClient.PlayGamesPlayer


func initialize() -> Result:
	PlayGamesSignInClient.is_authenticated()
	var is_authenticated = await PlayGamesSignInClient.user_authenticated
	if not is_authenticated:
		PlayGamesSignInClient.sign_in()
	var sign_in_success = await PlayGamesSignInClient.user_authenticated

	if not sign_in_success:
		return Result.Err("failed to sign in with Google Play Games")
	
	PlayGamesPlayersClient.load_current_player(true)
	player = await PlayGamesPlayersClient.current_player_loaded
	
	user_id = player.player_id
	user_name = player.display_name
	print("Logged in with Play Games as ", user_name)

	return Result.Ok(null)
