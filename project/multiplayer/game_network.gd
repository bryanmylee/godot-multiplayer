extends Node

const TIMEOUT := 5.0


func rpc_authority_with_return(
	event_emitter: Callable,
	event_opts: Variant,
	response_signal: Signal,
) -> Promise:
	return Promise.new(func (resolve, reject):
		var event_id := randi()
		event_emitter.rpc_id(Program.game_authority_id, event_id, event_opts)

		var response_handler := func (response_event_id: int, response_data: Variant):
			if response_event_id != event_id:
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
			+ "): timeout on rpc_authority_with_return(" + event_emitter.get_method() + ")"
		)
	)


func rpc_clients_except_id(
	except_id: int,
	event_emitter: Callable,
	event_opts: Variant,
) -> void:
	var other_ids := Program.server.clients.values() \
		.filter(func (c): return c.rtc_ready and c.id != Program.game_authority_id and c.id != except_id) \
		.map(func (c): return c.id)
	for to_peer_id in other_ids:
		event_emitter.rpc_id(to_peer_id, event_opts)
