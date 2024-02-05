extends Node
class_name PlayerId

@onready var id := get_multiplayer_authority()
@onready var is_local_player := is_multiplayer_authority()


signal platform_changed(platform: String)
var platform := "Windows" :
	set(new):
		platform = new
		platform_changed.emit(platform)


func _ready() -> void:
	if is_local_player:
		platform = ["Windows", "iOS", "Android", "Web"].pick_random()
