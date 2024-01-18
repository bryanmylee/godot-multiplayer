extends Node3D
class_name GameWorld


func _enter_tree() -> void:
	Program.world = self


func _ready():
	spawn_player()


#region Player Spawning
"""
type SpawnPlayerOptions = {
	scene_path?: String;
	position?: Vector3;
}
"""
func spawn_player(_opts: Dictionary = {}) -> void:
	"""
	@param opts: SpawnPlayerOptions
	"""
	pass
#endregion
