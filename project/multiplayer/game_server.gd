extends Node
class_name GameServer

enum MessageType {
	CONNECTED_TO_GAME_SERVER,
	WEBRTC_OFFER,
	WEBRTC_ANSWER,
	WEBRTC_CANDIDATE,
	WEBRTC_ADD_PEER,
	SET_GAME_AUTHORITY_ID,
}

const _DEFAULT_PORT := 8910
var env_port := OS.get_environment("PORT")
var port := int(env_port) if env_port else _DEFAULT_PORT

var env_id := OS.get_environment("SERVER_ID")
var id := int(env_id) if env_id else randi()

const _DEFAULT_TIMEOUT := 5.0
var env_timeout := OS.get_environment("SERVER_TIMEOUT")
var timeout := float(env_timeout) if env_timeout else _DEFAULT_TIMEOUT


func _enter_tree() -> void:
	# For prototyping, we usually want one client to simultaneously run
	# the server and client.
	if Program.is_dedicated_server or OS.is_debug_build():
		Program.server = self
	else:
		queue_free()


var socket := WebSocketMultiplayerPeer.new()
"Dict<{
	id: int;
	rtc_ready: bool;
}>"
var clients := {}


func _ready() -> void:
	socket.peer_connected.connect(_handle_client_connected)
	socket.peer_disconnected.connect(_handle_client_disconnected)
	Program.client.created_webrtc_mesh.connect(_handle_authority_webrtc_client_ready)

	var start_result := start()
	if start_result.is_err():
		# For a dedicated server, if we fail to start the server, we want to
		# exit the program with an error code so that the matchmaking server
		# can detect the failure.
		if Program.is_dedicated_server:
			OS.kill(OS.get_process_id())
		# For prototyping, `_ready` is called by all clients but only one client
		# will be able to bind to the listening port. Therefore, we can free all
		# programs that failed to start the server and assume they are just clients.
		elif OS.is_debug_build():
			queue_free()


func start() -> Result:
	Logger.server_log(["starting server on: ", port], ["init"])
	var result := Result.from_gderr(socket.create_server(port))
	if result.is_err():
		Logger.server_log(["failed to start server due to: ", result.unwrap_err()], ["init"])
	else:
		Logger.server_log(["started server on port: ", port], ["init"])
	return result


func _handle_client_connected(peer_id: int) -> void:
	Logger.server_log(["client connected: ", peer_id], ["client-server"])
	clients[str(peer_id)] = {
		"id": peer_id,
		"rtc_ready": false,
	}
	await message_peer(peer_id, MessageType.CONNECTED_TO_GAME_SERVER, peer_id).settled

	if Program.world != null:
		var spawn_result := Program.world.authority_spawn_player({
			"player_id": peer_id,
		})
		if spawn_result.is_err():
			Logger.server_log([spawn_result])
		else:
			Logger.server_log(["spawned player: ", peer_id])


func _handle_client_disconnected(peer_id: int) -> void:
	Logger.server_log(["client disconnected: ", peer_id], ["client-server"])
	
	if Program.world != null:
		var unspawn_result := Program.world.authority_unspawn_player(peer_id)
		Logger.server_log([unspawn_result])


func _handle_authority_webrtc_client_ready(peer_id: int) -> void:
	Logger.server_log(["authority client ready with id: ", peer_id], ["webrtc"])
	Program.game_authority_id = peer_id
	GameNetwork.game_network_ready.emit()
	
	await set_game_authority_on_existing_peers(Program.game_authority_id).settled
	Program.client.load_world()


func set_game_authority_on_existing_peers(authority_id: int) -> Promise:
	var ready_client_ids := clients.values() \
		.filter(func (c): return c.rtc_ready) \
		.map(func (c): return c.id)
	var set_authority_id_promises: Array[Promise] = []
	for client_id in ready_client_ids:
		set_authority_id_promises.append(
			message_peer(client_id, MessageType.SET_GAME_AUTHORITY_ID, authority_id)
		)
	return Promise.all(set_authority_id_promises)


#region Client-Server Communication
func _process(_delta: float) -> void:
	socket.poll()
	_read_incoming_packets()


func _read_incoming_packets() -> void:
	if socket.get_available_packet_count() == 0:
		return
	var packet = socket.get_packet()
	if packet == null:
		return
	var data_string = packet.get_string_from_utf8()
	var data: Dictionary = JSON.parse_string(data_string)
	if data.has("result"):
		_handle_client_response(data)
	else:
		_handle_client_message(data)


"Dict<{
	resolve(data): void,
	reject(err): void
}>"
var _message_response_handlers_for_id := {}
func _handle_client_response(message: Variant) -> void:
	"""
	@param message: ClientResponse
	"""
	if not message.has("id"):
		Logger.server_log(["received response without message id"], ["client-server"])
		return
	if not _message_response_handlers_for_id.has(message.id):
		Logger.server_log([
			"received response from client(", message.peer_id, ") with non-existent message id: ", message.id
		], ["client-server"])
		return
	var resolve_reject = _message_response_handlers_for_id[message.id]
	var result := Result.from_dict(message.result)
	if result.is_ok():
		resolve_reject.resolve.call(result.unwrap())
	else:
		resolve_reject.reject.call(result.unwrap_err())
	_message_response_handlers_for_id.erase(message.id)


func _handle_client_message(message: Variant) -> void:
	"""
	@param message: ClientMessage
	"""
	if message.mtype == GameClient.MessageType.WEBRTC_OFFER:
		var result: Result = await _forward_webrtc_offer(message.data.target_id, message.data).settled
		_respond_to_peer(message, result)
	elif message.mtype == GameClient.MessageType.WEBRTC_ANSWER:
		var result: Result = await _forward_webrtc_answer(message.data.target_id, message.data).settled
		_respond_to_peer(message, result)
	elif message.mtype == GameClient.MessageType.WEBRTC_CANDIDATE:
		var result: Result = await _forward_ice_candidate(message.data.target_id, message.data).settled
		_respond_to_peer(message, result)
	elif message.mtype == GameClient.MessageType.WEBRTC_READY:
		var result: Result = await _handle_webrtc_ready(message.peer_id)
		_respond_to_peer(message, result)


"""
type ServerMessage = {
	id: String;
	mtype: MessageType;
	data: Variant;
}
"""
func message_peer(peer_id: int, mtype: MessageType, data: Variant) -> Promise:
	return Promise.new(func(resolve, reject):
		var message_id := str(randi())
		_send_data_to_peer(peer_id, {
			"id": message_id,
			"mtype": mtype,
			"data": data,
		})
		_message_response_handlers_for_id[message_id] = {
			"resolve": resolve,
			"reject": reject,
		}
		await get_tree().create_timer(timeout).timeout
		reject.call(
			"server(" + str(id) \
			+ "): timeout on message(" + message_id \
			+ ") for peer(" + str(peer_id) + ")"
		)
		_message_response_handlers_for_id.erase(message_id)
	)


"""
type ServerResponse = {
	id: String;
	result: Result::to_dict;
}
"""
func _respond_to_peer(message: Variant, result: Result) -> void:
	"""
	@param message: ClientMessage
	"""
	_send_data_to_peer(message.peer_id, {
		"id": message.id,
		"result": result.to_dict(),
	})


func _send_data_to_peer(peer_id: int, data: Variant) -> void:
	var data_bytes := JSON.stringify(data).to_utf8_buffer()
	socket.get_peer(peer_id).put_packet(data_bytes)
#endregion


#region WebRTC Signalling
"""
type WebRTCAddPeerPayload = {
	target_id: int;
	to_offer: bool;
}
"""
func _handle_webrtc_ready(from_peer_id: int) -> Result:
	var other_ids := clients.values() \
		.filter(func (c): return c.rtc_ready) \
		.map(func (c): return c.id)
	Logger.server_log(["adding client(", from_peer_id, ") to peers: ", other_ids], ["client-server"])
	clients[str(from_peer_id)].rtc_ready = true

	await message_peer(from_peer_id, MessageType.SET_GAME_AUTHORITY_ID, Program.game_authority_id).settled

	var add_self_to_others_promises: Array[Promise] = []
	for to_peer_id in other_ids:
		add_self_to_others_promises.append(
			message_peer(to_peer_id, MessageType.WEBRTC_ADD_PEER, {
				"target_id": from_peer_id,
				"to_offer": false,
			})
		)
	await Promise.all(add_self_to_others_promises).settled

	var add_others_to_self_promises: Array[Promise] = []
	for to_peer_id in other_ids:
		add_others_to_self_promises.append(
			message_peer(from_peer_id, MessageType.WEBRTC_ADD_PEER, {
				"target_id": to_peer_id,
				"to_offer": true,
			})
		)
	return await Promise.all(add_others_to_self_promises).settled


func _forward_webrtc_offer(target_id: int, data: Variant) -> Promise:
	"""
	@param data: WebRTCOfferPayload
	"""
	return message_peer(target_id, MessageType.WEBRTC_OFFER, data)


func _forward_webrtc_answer(target_id: int, data: Variant) -> Promise:
	"""
	@param data: WebRTCAnswerPayload
	"""
	return message_peer(target_id, MessageType.WEBRTC_ANSWER, data)


func _forward_ice_candidate(target_id: int, data: Variant) -> Promise:
	"""
	@param data: ICECandidatePayload
	"""
	return message_peer(target_id, MessageType.WEBRTC_CANDIDATE, data)
#endregion
