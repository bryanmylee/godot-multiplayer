extends Node
class_name GameClient

enum MessageType {
	WEBRTC_OFFER,
	WEBRTC_ANSWER,
	WEBRTC_CANDIDATE,
	WEBRTC_READY,
}

const _DEFAULT_TIMEOUT := 5.0
var env_timeout := OS.get_environment("CLIENT_TIMEOUT")
var timeout := float(env_timeout) if env_timeout else _DEFAULT_TIMEOUT


func _enter_tree() -> void:
	Program.client = self


var server_socket := WebSocketMultiplayerPeer.new()
var rtc_network := WebRTCMultiplayerPeer.new()


func _ready() -> void:
	multiplayer.peer_connected.connect(_handle_webrtc_peer_connected)
	multiplayer.peer_disconnected.connect(_handle_webrtc_peer_connected)

	connect_to_game_server("127.0.0.1", 8910)
	await connected_to_game_server
	await create_webrtc_mesh().settled
	await get_tree().create_timer(5).timeout
	ping_network.rpc()


func _exit_tree() -> void:
	multiplayer.peer_connected.disconnect(_handle_webrtc_peer_connected)
	multiplayer.peer_disconnected.disconnect(_handle_webrtc_peer_connected)


func _handle_webrtc_peer_connected(new_peer_id: int) -> void:
	print("client(", peer_id, "): connected peer: ", new_peer_id)


func _handle_webrtc_peer_disconnected(disconnected_peer_id: int) -> void:
	print("client(", peer_id, "): disconnected peer: ", disconnected_peer_id)


var peer_id := 0


#region Client-Server Communication
func _process(_delta: float) -> void:
	server_socket.poll()
	_read_incoming_packets()


func _read_incoming_packets() -> void:
	if server_socket.get_available_packet_count() == 0:
		return
	var packet = server_socket.get_packet()
	if packet == null:
		return
	var data_string = packet.get_string_from_utf8()
	var data: Dictionary = JSON.parse_string(data_string)
	if data.has("result"):
		_handle_server_response(data)
	else:
		_handle_server_message(data)


"Record<String, { resolve(data): void, reject(err): void }>"
var _message_response_handlers_for_id := {}
func _handle_server_response(message: Variant) -> void:
	"""
	@param message: ServerResponse
	"""
	if not message.has("id"):
		print("client(", peer_id, "): received response without message id")
		return
	if not _message_response_handlers_for_id.has(message.id):
		print(
			"client(", peer_id,
			"): received response with invalid message id: ",
			message.id,
		)
		return
	var resolve_reject = _message_response_handlers_for_id[message.id]
	var result := Result.from_dict(message.result)
	if result.is_ok():
		resolve_reject.resolve.call(result.unwrap())
	else:
		resolve_reject.reject.call(result.unwrap_err())
	_message_response_handlers_for_id.erase(message.id)


func _handle_server_message(message: Variant) -> void:
	"""
	@param message: ServerMessage
	"""
	if message.mtype == GameServer.MessageType.CONNECTED_TO_GAME_SERVER:
		var result: Result = _handle_connected_to_server(message.data)
		_respond_to_server(message, result)
	elif message.mtype == GameServer.MessageType.WEBRTC_OFFER:
		var result: Result = _handle_webrtc_offer(message.data)
		_respond_to_server(message, result)
	elif message.mtype == GameServer.MessageType.WEBRTC_ANSWER:
		var result: Result = _handle_webrtc_answer(message.data)
		_respond_to_server(message, result)
	elif message.mtype == GameServer.MessageType.WEBRTC_CANDIDATE:
		var result: Result = _handle_ice_candidate(message.data)
		_respond_to_server(message, result)
	elif message.mtype == GameServer.MessageType.WEBRTC_ADD_PEER:
		var result: Result = await _handle_webrtc_add_peer(message.data)
		_respond_to_server(message, result)

"""
type ClientMessage = {
	id: String;
	peer_id: String;
	mtype: MessageType;
	data: Variant;
}
"""
func message_server(mtype: MessageType, data: Variant) -> Promise:
	return Promise.new(
		func(resolve, reject):
			var message_id := str(randi())
			_send_data_to_server({
				"id": message_id,
				"peer_id": peer_id,
				"mtype": mtype,
				"data": data,
			})
			_message_response_handlers_for_id[message_id] = {
				"resolve": resolve,
				"reject": reject,
			}
			await get_tree().create_timer(timeout).timeout
			reject.call(
				"client(" + str(peer_id) \
				+ "): timeout on message(" + message_id + ")"
			)
			_message_response_handlers_for_id.erase(message_id)
	)


"""
type ClientResponse = {
	id: String;
	peer_id: String;
	result: Result;
}
"""
func _respond_to_server(message: Variant, result: Result) -> void:
	"""
	@param message: ServerMessage
	"""
	_send_data_to_server({
		"peer_id": peer_id,
		"id": message.id,
		"result": result.to_dict(),
	})


func _send_data_to_server(data: Variant) -> void:
	var data_bytes := JSON.stringify(data).to_utf8_buffer()
	server_socket.put_packet(data_bytes)
#endregion


#region WebRTC Signalling
const WEBRTC_CONFIG = {
	"iceServers": [
		{
			# This STUN server is provided by Google free-of-charge for testing.
			"urls": ["stun:stun.l.google.com:19302"],
		},
	],
}


func _handle_webrtc_add_peer(data: Variant) -> Result:
	"""
	@param WebRTCAddPeerPayload
	"""
	if data.target_id == peer_id:
		return Result.Ok(null)
	
	var rtc_connection := WebRTCPeerConnection.new()
	var initialize_result := Result.from_gderr(
		rtc_connection.initialize(WEBRTC_CONFIG)
	)
	if initialize_result.is_err():
		print("client(", peer_id, "): failed to initialize RTC connection to peer: ", data.target_id)
		return initialize_result
	
	print("client(", peer_id, "): adding peer: ", data.target_id)

	var handle_session_description := Promise.new(
		func(resolve, reject):
			var description = await rtc_connection.session_description_created
			var type: String = description[0]
			var desc_data: Variant = description[1]
			var res := await set_local_description(type, desc_data, data.target_id)
			if res.is_err():
				reject.call(res.unwrap_err())
			else:
				resolve.call(res.unwrap())
	)

	var handle_ice_candidate := Promise.new(
		func(resolve, reject):
			var candidate = await rtc_connection.ice_candidate_created
			var media: String = candidate[0]
			var index: int = candidate[1]
			var sdp_name: String = candidate[2]
			var res := await send_ice_candidate(media, index, sdp_name, data.target_id)
			if res.is_err():
				reject.call(res.unwrap_err())
			else:
				resolve.call(res.unwrap())
	)

	rtc_network.add_peer(rtc_connection, data.target_id)

	if not data.to_offer:
		return Result.Ok(null)
	
	# If `create_offer` succeeds, the `session_description_created` and
	# `ice_candidate_created` signals are emitted.
	var offer_result := Result.from_gderr(rtc_connection.create_offer())
	if offer_result.is_err():
		print("client(", peer_id, "): ", offer_result.to_string())
		return offer_result
	
	var connection_ready_result: Result = await Promise.all([
		handle_session_description,
		handle_ice_candidate,
	]).settled
	if connection_ready_result.is_err():
		print("client(", peer_id, "): ", connection_ready_result.to_string())
		return connection_ready_result

	print("client(", peer_id, "): added peer: ", data.target_id)
	return Result.Ok(null)


func _handle_webrtc_offer(data: Variant) -> Result:
	"""
	@param data: WebRTCOfferPayload
	"""
	print("client(", peer_id, "): received an offer from: ", data.sender_id)
	if not rtc_network.has_peer(data.sender_id):
		var err := Result.Err("failed to find offering peer: " + str(data.sender_id))
		print("client(", peer_id, "): ", err.to_string())
		return err
	return Result.from_gderr(
		rtc_network.get_peer(data.sender_id).connection.set_remote_description(
			"offer",
			data.offer
		)
	)


func _handle_webrtc_answer(data: Variant) -> Result:
	print("client(", peer_id, "): received an answer from: ", data.sender_id)
	if not rtc_network.has_peer(data.sender_id):
		var err := Result.Err("failed to find answering peer: " + str(data.sender_id))
		print("client(", peer_id, "): ", err.to_string())
		return err
	return Result.from_gderr(
		rtc_network.get_peer(data.sender_id).connection.set_remote_description(
			"answer",
			data.answer
		)
	)


func _handle_ice_candidate(data: Variant) -> Result:
	print("client(", peer_id, "): received an ICE candidate from: ", data.sender_id)
	if not rtc_network.has_peer(data.sender_id):
		var err := Result.Err("failed to find ICE candidate peer: " + str(data.sender_id))
		print("client(", peer_id, "): ", err.to_string())
		return err
	return Result.from_gderr(
		rtc_network.get_peer(data.sender_id).connection.add_ice_candidate(
			data.media,
			data.index,
			data.sdp,
		)
	)


"""
type WebRTCOfferPayload = {
	offer: String;
	sender_id: int;
	target_id: int;
}
"""
func send_webrtc_offer(offer: Variant, target_id: int) -> Result:
	print("client(", peer_id, "): sending offer to target: ", target_id)
	var result: Result = await message_server(MessageType.WEBRTC_OFFER, {
		"offer": offer,
		"sender_id": peer_id,
		"target_id": target_id,
	}).settled
	if result.is_err():
		print("client(", peer_id, "): failed to send offer to target: ", target_id)
	else:
		print("client(", peer_id, "): successfully sent offer to target: ", target_id)
	return result


"""
type WebRTCAnswerPayload = {
	answer: String;
	sender_id: int;
	target_id: int;
}
"""
func send_webrtc_answer(answer: Variant, target_id: int) -> Result:
	print("client(", peer_id, "): sending answer to target: ", target_id)
	var result: Result = await message_server(MessageType.WEBRTC_ANSWER, {
		"answer": answer,
		"sender_id": peer_id,
		"target_id": target_id,
	}).settled
	if result.is_err():
		print("client(", peer_id, "): failed to send answer to target: ", target_id)
	else:
		print("client(", peer_id, "): successfully sent answer to target: ", target_id)
	return result


func set_local_description(type: String, data: Variant, target_id: int) -> Result:
	# Make sure a connection has been established for the offer target.
	if not rtc_network.has_peer(target_id):
		var err := Result.Err("Failed to find peer " + str(target_id) + " to set local description for")
		print("client(", peer_id, "): ", err.to_string())
		return err
	
	print("client(", peer_id, "): setting local description for ", target_id)
	var set_desc_result := Result.from_gderr(
		rtc_network.get_peer(target_id).connection.set_local_description(type, data)
	)
	if set_desc_result.is_err():
		print("client(", peer_id, "): ", set_desc_result.to_string())
		return set_desc_result
	
	if type == "offer":
		return await send_webrtc_offer(data, target_id)
	else:
		return await send_webrtc_answer(data, target_id)


"""
type ICECandidatePayload = {
	media: String;
	index: int;
	sdp: String;
	sender_id: int;
	target_id: int;
}
"""
func send_ice_candidate(
	media: String,
	index: int,
	sdp: String,
	target_id: int
) -> Result:
	var result: Result = await message_server(MessageType.WEBRTC_CANDIDATE, {
		"media": media,
		"index": index,
		"sdp": sdp,
		"sender_id": peer_id,
		"target_id": target_id,
	}).settled
	if result.is_err():
		print("client(", peer_id, "): failed to send ICE candidate to target: ", target_id)
	else:
		print("client(", peer_id, "): successfully sent ICE candidate to target: ", target_id)
	return result
#endregion


signal connected_to_game_server(assigned_id: int)
func connect_to_game_server(host: String, port: int) -> void:
	var protocol := "wss://" if Program.ssl_enabled else "ws://"
	var address := protocol + host + ":" + str(port)
	print("client(...): connecting to game server at: ", address)
	server_socket.create_client(address)


func _handle_connected_to_server(assigned_id: int) -> Result:
	peer_id = assigned_id
	print("client(", peer_id, "): connected to game server")
	connected_to_game_server.emit(assigned_id)
	return Result.Ok(assigned_id)


func create_webrtc_mesh() -> Promise:
	var create_mesh_result := Result.from_gderr(rtc_network.create_mesh(peer_id))
	if create_mesh_result.is_err():
		print("client(", peer_id, "): failed to create RTC mesh")
		return Promise.new(func (_resolve, reject): reject.call("failed to create RTC mesh"))
	print("client(", peer_id, "): created RTC mesh")
	multiplayer.multiplayer_peer = rtc_network
	
	return message_server(MessageType.WEBRTC_READY, null)


@rpc("any_peer")
func ping_network() -> void:
	print("client(", peer_id, "): ping from ", multiplayer.get_remote_sender_id())
