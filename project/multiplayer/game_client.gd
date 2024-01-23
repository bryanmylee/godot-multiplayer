extends Node
class_name GameClient

const _DEFAULT_TIMEOUT := 5.0
var env_timeout := OS.get_environment("CLIENT_TIMEOUT")
var timeout := float(env_timeout) if env_timeout else _DEFAULT_TIMEOUT

const _DEFAULT_SERVER_HOST := "127.0.0.1"
var env_server_host := OS.get_environment("SERVER_HOST")
var server_host := env_server_host if env_server_host else _DEFAULT_SERVER_HOST

const _DEFAULT_SERVER_PORT := 8910
var env_server_port := OS.get_environment("SERVER_PORT")
var server_port := int(env_server_port) if env_server_port else _DEFAULT_SERVER_PORT

## We use this in place of `multiplayer.get_unique_id()` for more customization and static access via `Program.client`.
var peer_id := 0


func _enter_tree() -> void:
	Program.client = self
