extends Node
class_name IdentityProvider

var id:
	get: return get_parent().name.to_int()
var is_local_player:
	get: return Program.client.peer_id == id
