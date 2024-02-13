extends Node

enum ProviderName {
	STEAM = 0,
	GOOGLE_PLAY_GAMES = 1,
	APPLE_GAME_CENTER = 2,
	WEB_OAUTH2 = 3,
}

const PROVIDER_SCRIPT := {
	ProviderName.STEAM: "res://authentication/providers/steam_auth_provider.gd",
	ProviderName.GOOGLE_PLAY_GAMES: "res://authentication/providers/google_play_games_auth_provider.gd",
	ProviderName.APPLE_GAME_CENTER: "res://authentication/providers/apple_game_center_auth_provider.gd",
	ProviderName.WEB_OAUTH2: "res://authentication/providers/web_oauth2_auth_provider.gd",
}

var providers_node: Node

var providers: Array[AuthProvider] :
	get:
		var _providers: Array[AuthProvider] = []
		_providers.assign(providers_node.get_children())
		return _providers

var main_provider: AuthProvider :
	get:
		return providers.front() if providers.size() > 0 else null

## [codeblock]
## Option<String>
## [/codeblock]
var user_id := Option.None()
## [codeblock]
## Option<String>
## [/codeblock]
var user_name := Option.None()
## [codeblock]
## Option<String>
## [/codeblock]
var access_token := Option.None()
## [codeblock]
## Option<String>
## [/codeblock]
var refresh_token := Option.None()

const ACCESS_TOKEN_EXPIRY_BUFFER_SEC := 20
## [codeblock]
## Option<String>
## [/codeblock]
var access_token_expires_at := Option.None() :
	set(new):
		access_token_expires_at = new
		if access_token_expires_at.is_none():
			return
		refresh_token_timer_node.stop()
		var expiry_ts := Time.get_unix_time_from_datetime_string(access_token_expires_at.unwrap())
		var now_ts = ceili(Time.get_unix_time_from_system())
		var expires_in_sec = expiry_ts - now_ts - ACCESS_TOKEN_EXPIRY_BUFFER_SEC
		if expires_in_sec <= 0:
			refresh_access_token()
		else:
			refresh_token_timer_node.wait_time = expires_in_sec
			refresh_token_timer_node.start()

var refresh_token_timer_node: Timer
var is_token_refreshing := false


func _ready() -> void:
	setup_providers_node()
	setup_refresh_token_timer_node()


func setup_providers_node() -> void:
	providers_node = Node.new()
	providers_node.name = "Providers"
	add_child(providers_node)


func setup_refresh_token_timer_node() -> void:
	refresh_token_timer_node = Timer.new()
	refresh_token_timer_node.name = "RefreshTokenTimer"
	refresh_token_timer_node.one_shot = true
	add_child(refresh_token_timer_node)
	refresh_token_timer_node.timeout.connect(refresh_access_token)


## [codeblock]
## @returns Result<AuthProvider>
## [/codeblock]
func initialize_main_provider() -> Result:
	if main_provider != null:
		return Result.Err("main authentication provider already initialized")
	
	match OS.get_name():
		"Windows", "macOS", "Linux", "FreeBSD", "NetBSD", "OpenBSD", "BSD":
			return await add_provider(ProviderName.STEAM)
		"Android":
			return await add_provider(ProviderName.GOOGLE_PLAY_GAMES)
		"iOS":
			return await add_provider(ProviderName.APPLE_GAME_CENTER)
		"Web":
			return await add_provider(ProviderName.WEB_OAUTH2)
		_:
			return Result.Err("no matching platform")


## [codeblock]
## @returns Result<AuthProvider>
## [/codeblock]
func add_provider(pname: ProviderName) -> Result:
	var script = PROVIDER_SCRIPT[pname]
	var provider := load(script).new() as AuthProvider
	providers_node.add_child(provider, true)
	provider.owner = providers_node
	
	var init_result := await provider.initialize()
	if init_result.is_err():
		providers_node.remove_child(provider)
		return init_result

	return Result.Ok(provider)


## [codeblock]
## @returns Result<null>
## [/codeblock]
func sign_in() -> Result:
	if user_id.is_some():
		return Result.Ok(null)
	
	if main_provider == null:
		return Result.Err("main authentication provider not yet initialized")
	
	var sign_in_result := await main_provider.server_sign_in()
	if sign_in_result.is_err():
		return sign_in_result
	
	var sign_in_body = sign_in_result.unwrap()
	match sign_in_body.type:
		"success":
			print("Successfully logged in: ", sign_in_body.payload)
			user_id = Option.new(sign_in_body.payload.user.id)
			user_name = Option.new(sign_in_body.payload.user.name)
			access_token = Option.new(sign_in_body.payload.access_token.value)
			refresh_token = Option.new(sign_in_body.payload.refresh_token.value)

			access_token_expires_at = Option.new(
				sign_in_body.payload.access_token.expires_at
			)
		"pending_link_or_create":
			print("Possible existing account: ", sign_in_body.payload)
	
	return Result.Ok(null)


const AUTH_SERVER_REFRESH_PATH := "/auth/refresh"
func refresh_access_token() -> Result:
	if refresh_token.is_none():
		return Result.Err("cannot refresh access token without refresh token")
	
	is_token_refreshing = true
	var request_result: Result = await HTTPUtils.fetch(
		Program.AUTH_SERVER_URI + AUTH_SERVER_REFRESH_PATH,
		["Content-Type: application/json"],
		HTTPClient.METHOD_POST,
		JSON.stringify({ "refresh_token": refresh_token.unwrap() })
	).settled

	if request_result.is_err():
		push_error(request_result.unwrap_err())
		is_token_refreshing = false
		return request_result
	
	var response = request_result.unwrap()
	if response.response_code != HTTPClient.RESPONSE_OK:
		is_token_refreshing = false
		return Result.Err("failed to refresh access token: %s" % response.response_code)
	
	var body_text: String = response.body.get_string_from_utf8()
	var body = JSON.parse_string(body_text)
	access_token = Option.Some(body.access_token.value)
	refresh_token = Option.Some(body.refresh_token.value)

	access_token_expires_at = Option.new(body.access_token.expires_at)

	is_token_refreshing = false
	return Result.Ok(null)
