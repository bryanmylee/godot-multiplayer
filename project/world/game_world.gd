extends Node3D
class_name GameWorld


func _enter_tree() -> void:
  Program.world = self


func _ready():
  pass


#region Game Logic
const DEFAULT_PLAYER_PATH := "res://player/player.tscn"
func spawn_player(player_path: String = DEFAULT_PLAYER_PATH) -> void:
  pass
#endregion