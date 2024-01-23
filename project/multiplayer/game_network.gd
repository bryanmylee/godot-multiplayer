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
			"client(" + str(Program.client.peer_id) \
			+ "): timeout on rpc_id_with_return(" + rpc_fn.get_method() + ")"
		)
	)


func rpc_clients_except_id(
	except_id: int,
	rpc_fn: Callable,
	arg1: Variant = null,
	arg2: Variant = null,
	arg3: Variant = null,
) -> void:
	var other_ids := Program.server.clients.values() \
		.filter(func (c): return c.rtc_ready and c.id != Program.game_authority_id and c.id != except_id) \
		.map(func (c): return c.id)
	for to_peer_id in other_ids:
		if arg1 == null:
			rpc_fn.rpc_id(to_peer_id)
		elif arg2 == null:
			rpc_fn.rpc_id(to_peer_id, arg1)
		elif arg3 == null:
			rpc_fn.rpc_id(to_peer_id, arg1, arg2)
		else:
			rpc_fn.rpc_id(to_peer_id, arg1, arg2, arg3)
#endregion


# Emitted when the RTC network mesh is configured and `Program.game_authority_id` is properly configured.
signal game_network_ready


func _ready() -> void:
	game_network_ready.connect(NetworkTime.start)
