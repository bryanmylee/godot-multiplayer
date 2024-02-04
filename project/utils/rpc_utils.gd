extends Node


#region RPC Helpers
## [codeblock]
## @returns Promise<T, String>
## [/codeblock]
const TIMEOUT := 5.0
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


static var EMPTY_ARG = Symbol.new()
func rpc_except_id(
	except_id: int,
	rpc_fn: Callable,
	arg1: Variant = EMPTY_ARG,
	arg2: Variant = EMPTY_ARG,
) -> void:
	# Avoid doing arg checks repeatedly for all peers.
	var runner := (func (id: int): rpc_fn.rpc_id(id)) if EMPTY_ARG.is_equal(arg1) \
		else (func (id: int): rpc_fn.rpc_id(id, arg1)) if EMPTY_ARG.is_equal(arg2) \
		else (func (id: int): rpc_fn.rpc_id(id, arg1, arg2))
	
	var peer_ids := multiplayer.get_peers()
	for peer_id in peer_ids:
		if peer_id == except_id:
			continue
		runner.call(peer_id)
#endregion
