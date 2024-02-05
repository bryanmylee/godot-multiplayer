extends Node

enum ProviderName {
	STEAM = 0,
	GOOGLE_PLAY_GAMES = 1,
	APPLE_GAME_CENTER = 2,
	WEB_OAUTH2 = 3,
}

const PROVIDER_SCRIPT := {
	ProviderName.STEAM: "res://authentication/providers/steam_authentication_provider.gd",
	ProviderName.GOOGLE_PLAY_GAMES: "res://authentication/providers/google_play_games_authentication_provider.gd",
	ProviderName.APPLE_GAME_CENTER: "res://authentication/providers/apple_game_center_authentication_provider.gd",
	ProviderName.WEB_OAUTH2: "res://authentication/providers/web_oauth2_authentication_provider.gd",
}

var providers_node: Node

var providers: Array[AuthenticationProvider] :
	get:
		var _providers: Array[AuthenticationProvider] = []
		_providers.assign(providers_node.get_children())
		return _providers

var main_provider: AuthenticationProvider :
	get:
		return providers.front() if providers.size() > 0 else null


func _ready() -> void:
	providers_node = Node.new()
	providers_node.name = "Providers"
	add_child(providers_node)


## [codeblock]
## @returns Result<AuthenticationProvider>
## [/codeblock]
func initialize_default() -> Result:
	if main_provider != null:
		return Result.Err("Main authentication provider already initialized")
	
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
			return Result.Err("No matching platform")


## [codeblock]
## @returns Result<AuthenticationProvider>
## [/codeblock]
func add_provider(pname: ProviderName) -> Result:
	var script = PROVIDER_SCRIPT[pname]
	var provider := load(script).new() as AuthenticationProvider
	providers_node.add_child(provider, true)
	provider.owner = providers_node
	
	var init_result := await provider.initialize()
	if init_result.is_err():
		providers_node.remove_child(provider)
		return init_result
	
	return Result.Ok(provider)
