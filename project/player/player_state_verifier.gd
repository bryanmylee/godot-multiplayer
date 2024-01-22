extends NetworkVerifier
class_name PlayerStateVerifier

func _enter_tree() -> void:
	set_multiplayer_authority(Program.game_authority_id)


@export_group("Dependencies")
@export var player: Player


#region Verified State
# const MIN_MAX_POSITION_DRIFT := 0.1
# const POSITION_DRIFT_FACTOR := 1.25
# const POSITION_DRIFT_TICKS_BEFORE_RESET := 5
# var position_drift_ticks := 0
# var position: Vector3

# @rpc
# func verify_position(check: Vector3) -> void:
# 	position = check

# 	if not id_provider.is_local_player:
# 		player.position = position
# 		return
	
# 	var allowed_drift := maxf(player.velocity.length(), MIN_MAX_POSITION_DRIFT) \
# 		* delta_verification * POSITION_DRIFT_FACTOR
# 	var has_drifted := player.position.distance_squared_to(position) > allowed_drift ** 2
# 	if has_drifted:
# 		position_drift_ticks += 1
# 		if position_drift_ticks >= POSITION_DRIFT_TICKS_BEFORE_RESET:
# 			Logger.client_log(["position drifted from authority by ", player.position.distance_to(position)])
# 			player.position = position
# 	else:
# 		position_drift_ticks = 0


"NetworkVerifiedState<Vector3>"
@onready var position := $position as NetworkVerifiedState


func _ready() -> void:
	position.authority_verified.connect(
		func (tick: int, value: Variant, history: HistoryRecorder):
			"""
			@param value: Vector3
			"""
			if not id_provider.is_local_player:
				player.position = value
				return
			
			var record = history.search(tick)
			if record == null:
				# print("no history record")
				player.position = value
				return
			var verified_value = record[0][1] if record.size() == 1 \
				else record[0][1].lerp(record[1][1], remap(tick, record[0][0], record[1][0], 0.0, 1.0))
			# Logger.client_log(["player(", id_provider.id, ") verified value: ", verified_value])
	)
#endregion
