extends Control
class_name SceneTransition

@onready var anim_player := $Overlay/AnimationPlayer as AnimationPlayer


func _ready() -> void:
	anim_player.play_backwards("fade")


func skip_to(next_node: Node) -> void:
	replace_screen_node(next_node)


func fade_to(next_node: Node, speed := 2.0) -> void:
	anim_player.play("fade", -speed, speed)
	await anim_player.animation_finished
	replace_screen_node(next_node)
	anim_player.play("fade", -speed, -speed, true)
	await anim_player.animation_finished


var curr_node: Node
func replace_screen_node(node: Node) -> void:
	if curr_node != null:
		$UI.remove_child(curr_node)
	curr_node = node
	$UI.add_child(node)
