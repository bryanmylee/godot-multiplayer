extends Node
class_name NetworkVerifiedState

"""
<TValue>
"""
signal authority_verified(tick: int, value: Variant, history: HistoryRecorder)


"HistoryRecorder<TValue>"
var history: HistoryRecorder


func _ready() -> void:
	"""
	@param initial: TValue
	"""
	history = HistoryRecorder.new()


func verify(value: Variant) -> void:
	"""
	@param value: TValue
	"""
	var tick := GameNetwork.get_network_synced_tick()
	if Program.is_game_authority:
		Logger.server_log(["tick: ", tick])
		authority_verify.rpc(tick, value)
	else:
		Logger.client_log(["tick: ", tick])
		history.append(tick, value)


@rpc
func authority_verify(tick: int, verified_value: Variant) -> void:
	authority_verified.emit(tick, verified_value, history)
