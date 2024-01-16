extends Node
class_name IdentityProvider

var multiplayer_id := 1
var is_remote_player :
	get: return multiplayer.get_unique_id() != multiplayer_id

