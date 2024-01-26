extends Node
class_name Authentication

enum ProviderName {
	STEAM = 0,
	GOOGLE_PLAY_GAMES = 1,
	APPLE_GAME_CENTER = 2,
	OPEN_ID = 3,
}

const PROVIDER_SCRIPT := {
	ProviderName.STEAM: "res://authentication/providers/steam_authentication_provider.gd",
	ProviderName.GOOGLE_PLAY_GAMES: "res://authentication/providers/google_play_games_authentication_provider.gd",
	ProviderName.APPLE_GAME_CENTER: "res://authentication/providers/apple_game_center_authentication_provider.gd",
	ProviderName.OPEN_ID: "res://authentication/providers/open_id_authentication_provider.gd",
}

@onready var providers_node := $Providers

var providers: Array[AuthenticationProvider] :
	get:
		var _providers: Array[AuthenticationProvider] = []
		_providers.assign(providers_node.get_children())
		return _providers

var main_provider: AuthenticationProvider :
	get:
		return providers[0]


func initialize_default() -> Result:
	match OS.get_name():
		"Windows", "macOS", "Linux", "FreeBSD", "NetBSD", "OpenBSD", "BSD":
			return add_provider(ProviderName.STEAM)
		"Android":
			return add_provider(ProviderName.GOOGLE_PLAY_GAMES)
		"iOS":
			return add_provider(ProviderName.APPLE_GAME_CENTER)
		"Web":
			return add_provider(ProviderName.OPEN_ID)
		_:
			return Result.Err("No matching platform")


func add_provider(pname: ProviderName) -> Result:
	"""
	@returns Result<AuthenticationProvider>
	"""
	var script = PROVIDER_SCRIPT[pname]
	var provider := load(script).new() as AuthenticationProvider
	
	var init_result := provider.initialize()
	if init_result.is_err():
		return init_result
	
	providers_node.add_child(provider, true)
	return Result.Ok(provider)
