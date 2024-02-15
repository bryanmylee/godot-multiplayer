class_name PlayGamesSignInClient
extends Node
## Client with sign in functionality.
##
## This autoload exposes methods and signals to control the player sign in process.[br]
## [br]
## If the player is already signed in into Google Play Games, the plugin does
## a check at startup, so usually you don't have to use these methods. Use them only 
## to provide a manual way for the user to sign in.

## Signal emitted after calling the [method is_authenticated] method.[br]
## [br]
## [param is_authenticated]: Indicates if the user is authenticated or not.
signal user_authenticated(is_authenticated: bool)

## Signal emitted after calling the [method request_server_side_access] method.[br]
## [br]
## [param token]: The OAuth 2.0 authorization code as a string.
signal server_side_access_requested(success: bool, token: String)

var core: PlayGamesServicesCore

func _init(_core: PlayGamesServicesCore):
	core = _core

func _ready() -> void:
	_connect_signals()

func _connect_signals() -> void:
	if core.android_plugin:
		core.android_plugin.userAuthenticated.connect(func(authenticated: bool):
			user_authenticated.emit(authenticated)
		)
		core.android_plugin.serverSideAccessRequested.connect(func(success: bool, token: String):
			server_side_access_requested.emit(success, token)
		)

## Use this method to check if the user is already authenticated. If the user is authenticated,
## a popup will be shown on screen.[br]
## [br]
## The method emits the [signal user_authenticated] signal.
func is_authenticated() -> void:
	if core.android_plugin:
		core.android_plugin.isAuthenticated()

## Use this method to provide a manual way to the user for signing in.[br]
## [br]
## The method emits the [signal user_authenticated] signal.
func sign_in() -> void:
	if core.android_plugin:
		core.android_plugin.signIn()

## Requests server-side access to Play Games Services for the currently signed-in player.
## When requested, an authorization code is returned that can be used by your server to exchange
## for an access token that can be used by your server to access the Play Games Services web APIs.[br]
## [br]
## The method emits the [signal server_side_access_requested] signal.[br]
## [br]
## [param server_client_id]: The client ID of the server that will perform the authorization code flow exchange.[br]
## [param force_refresh_token]: If true, when the returned authorization code is exchanged, a refresh
## token will be included in addition to an access token.
func request_server_side_access(server_client_id: String, force_refresh_token: bool) -> void:
	if core.android_plugin:
		core.android_plugin.requestServerSideAccess(server_client_id, force_refresh_token)
