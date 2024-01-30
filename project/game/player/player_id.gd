extends Node
class_name PlayerId

var id:
	get: return get_parent().name.to_int()
var is_local_player:
	get: return multiplayer.get_unique_id() == id


signal platform_changed(platform: String)
var platform := "Windows" :
	set(new):
		platform = new
		platform_changed.emit(platform)


@rpc("any_peer", "reliable")
func authority_set_platform(_platform: String):
	var sender_id := multiplayer.get_remote_sender_id()
	if sender_id != id:
		return
	platform = _platform


func _ready() -> void:
	if is_local_player:
		authority_set_platform.rpc_id(1, ["Windows", "iOS", "Android", "Web"].pick_random())
