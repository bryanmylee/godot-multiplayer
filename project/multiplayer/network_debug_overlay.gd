extends Control
class_name NetworkDebugOverlay

@onready var server_rtt_value := $Fields/ServerRTT/Value as Label


func _process(_delta: float) -> void:
	server_rtt_value.text = str(snapped(NetworkTime.remote_rtt * 1000, 0.01)) + "ms"
