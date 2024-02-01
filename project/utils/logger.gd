class_name Logger

## [codeblock]
## Dict<bool>
## [/codeblock]
const FILTERED_TAGS := {
	"webrtc": true,
}


static func client_log(args: Array, tags := []) -> void:
	if tags.any(func (t): return FILTERED_TAGS.get(t, false)):
		return
	
	var id_str := "client(" + str(Program.game_client.peer_id) + "): "
	
	var log_str := id_str + "".join(args)
	
	if not tags.is_empty():
		var tag_str := "".join(tags.map(func (t): return "[" + t + "]"))
		log_str = tag_str + " " + log_str
	
	print(log_str)


static func server_log(args: Array, tags := []) -> void:
	if tags.any(func (t): return FILTERED_TAGS.get(t, false)):
		return
	
	var id_str := "server(" + str(Program.game_server.id) + "): "
	
	var log_str := id_str + "".join(args)
	
	if not tags.is_empty():
		var tag_str := "".join(tags.map(func (t): return "[" + t + "]"))
		log_str = tag_str + " " + log_str
	
	print(log_str)
