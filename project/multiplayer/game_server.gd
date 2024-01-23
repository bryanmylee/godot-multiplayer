extends Node
class_name GameServer

const _DEFAULT_PORT := 19000
var env_port := OS.get_environment("PORT")
var port := int(env_port) if env_port else _DEFAULT_PORT

var env_id := OS.get_environment("SERVER_ID")
var id := int(env_id) if env_id else randi()

const _DEFAULT_TIMEOUT := 5.0
var env_timeout := OS.get_environment("SERVER_TIMEOUT")
var timeout := float(env_timeout) if env_timeout else _DEFAULT_TIMEOUT

"Dict<{
	id: int;
	rtc_ready: bool;
}>"
var clients := {}


func _enter_tree() -> void:
	# For prototyping, we usually want one client to simultaneously run
	# the server and client.
	if Program.is_dedicated_server or OS.is_debug_build():
		Program.server = self
	else:
		queue_free()


func start() -> Result:
	return Result.Err("missing implementation for `start`")
