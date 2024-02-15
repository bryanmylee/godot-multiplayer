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
	
	provider_type = "play_games"
	provider_id = Option.new(player.player_id)
	user_name = Option.new(player.display_name)
	print("Logged in with Play Games as ", user_name.unwrap())

	return Result.Ok(null)


const AUTH_SERVER_SIGN_IN_PATH := "/auth/play-games/sign-in"
func server_sign_in() -> Result:
	if provider_id.is_none():
		return Result.Err("not authenticated locally")
	
	google_play_games.core.sign_in_client.request_server_side_access(Program.AUTH_SERVER_PLAY_GAMES_OAUTH_CLIENT_ID, false)
	var auth_code_result = await google_play_games.core.sign_in_client.server_side_access_requested
	var token_success: bool = auth_code_result[0]
	if not token_success:
		return Result.Err(auth_code_result[1])
	var auth_code: String = auth_code_result[1]

	var request_result: Result = await HTTPUtils.fetch(
		Program.AUTH_SERVER_URI + AUTH_SERVER_SIGN_IN_PATH,
		["Content-Type: text/plain"],
		HTTPClient.METHOD_POST,
		auth_code,
	).settled
	
	if request_result.is_err():
		return request_result
	
	var response = request_result.unwrap()
	if response.response_code != HTTPClient.RESPONSE_OK:
		return Result.Err("failed to sign in: %s" % response.response_code)
	
	var body_text: String = response.body.get_string_from_utf8()
	return Result.Ok(JSON.parse_string(body_text))
