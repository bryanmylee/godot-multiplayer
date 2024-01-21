extends NetworkVerifier
class_name PlayerStateVerifier

func _enter_tree() -> void:
	set_multiplayer_authority(Program.game_authority_id)


@export_group("Dependencies")
@export var player: Player


#region Verified State
const POSITION_DRIFT_FACTOR := 1.25
const POSITION_DRIFT_TICKS_BEFORE_RESET := 2
var position_drift_ticks := 0
var position: Vector3 :
	set(new):
		position = new
		if Program.is_game_authority:
			verify_on_peers("position", position)
			return
		if not id_provider.is_local_player:
			player.position = position
			return
		
		var allowed_drift := player.velocity.length() * delta_verification * POSITION_DRIFT_FACTOR
		var has_drifted := player.position.distance_squared_to(position) > allowed_drift ** 2
		if has_drifted:
			position_drift_ticks += 1
			if position_drift_ticks >= POSITION_DRIFT_TICKS_BEFORE_RESET:
				Logger.client_log(["position drifted from authority"])
				player.position = position
		else:
			position_drift_ticks = 0
#endregion


func _ready() -> void:
	set_verification_process(Program.is_game_authority)


func _verification_process() -> void:
	position = player.position
