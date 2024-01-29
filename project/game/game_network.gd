extends Node

const TIMEOUT := 5.0


#region RPC Helpers
func rpc_id_with_return(
	peer_id: int,
	response_signal: Signal,
	rpc_fn: Callable,
	arg1: Variant,
) -> Promise:
	return Promise.new(func (resolve, reject):
		var id := randi()
		rpc_fn.rpc_id(peer_id, id, arg1)

		var response_handler := func (response_id: int, response_data: Variant):
			if response_id != id:
				return
			var response_result := Result.from_dict(response_data)
			if response_result.is_ok():
				resolve.call(response_result.unwrap())
			else:
				reject.call(response_result.unwrap_err())
		
		# Auto-disconnected when `response_handler` is deallocated.
		response_signal.connect(response_handler)
		
		await get_tree().create_timer(TIMEOUT).timeout
		reject.call(
			"client(" + str(Program.game_client.peer_id) \
			+ "): timeout on rpc_id_with_return(" + rpc_fn.get_method() + ")"
		)
	)
#endregion


signal game_network_ready


func _ready() -> void:
	game_network_ready.connect(NetworkTime.start)
