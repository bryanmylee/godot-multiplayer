extends AuthProvider
class_name GooglePlayGamesAuthProvider

@onready var google_play_games: GooglePlayGamesService = ServiceManager.get_service("GooglePlayGames")


func _init() -> void:
	name = "GooglePlayGames"


var player: PlayGamesPlayersClient.PlayGamesPlayer


func initialize() -> Result:
	google_play_games.core.sign_in_client.is_authenticated()
	var is_authenticated = await google_play_games.core.sign_in_client.user_authenticated
	if not is_authenticated:
		google_play_games.core.sign_in_client.sign_in()
		var sign_in_success = await google_play_games.core.sign_in_client.user_authenticated
		if not sign_in_success:
			return Result.Err("failed to sign in with Google Play Games")
	
	google_play_games.core.players_client.load_current_player(true)
	player = await google_play_games.core.players_client.current_player_loaded
	
	user_id = player.player_id
	user_name = player.display_name
	print("Logged in with Play Games as ", user_name)

	return Result.Ok(null)
