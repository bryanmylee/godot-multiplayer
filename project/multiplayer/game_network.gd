extends Node

const TIMEOUT := 5.0


func server_rpc(
	event_emitter: Callable,
	event_opts: Dictionary,
	response_signal: Signal
) -> Promise:
	return Promise.new(func (resolve, reject):
		var event_id := str(randi())
		event_emitter.rpc_id(get_multiplayer_authority(), event_id, event_opts)

		var response_handler := func (response_event_id: String, response_data: Variant):
			if response_event_id != event_id:
				return
			var response_result := Result.from_dict(response_data)
			if response_result.is_ok():
				resolve.call(response_result.unwrap())
			else:
				reject.call(response_result.unwrap_err())
			
		response_signal.connect(response_handler)

		await get_tree().create_timer(TIMEOUT).timeout
		reject.call(
			"client(" + str(multiplayer.get_unique_id()) \
			+ "): timeout on predictive event(" + event_id + ")"
		)
	)
