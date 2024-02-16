extends AuthProvider
class_name SteamAuthProvider


func _init() -> void:
	name = "Steam"


func initialize() -> Result:
	provider_type = "steam"
	provider_id = Option.new(str(Steam.getSteamID()))
	user_name = Option.new(Steam.getPersonaName())
	print("Logged in with Steam as ", user_name.unwrap())

	return Result.Ok(null)


const AUTH_SERVER_SIGN_IN_PATH := "/auth/steam/sign-in"
func server_sign_in() -> Result:
	Steam.getAuthTicketForWebApi(Program.AUTH_SERVER_STEAM_IDENTITY)
	var ticket_payload = await Steam.get_ticket_for_web_api

	var auth_ticket_result: Steam.Result = ticket_payload[1]
	if auth_ticket_result != Steam.RESULT_OK:
		push_error("failed to get auth ticket: %s" % auth_ticket_result)
		return Result.Err(auth_ticket_result)
	
	var auth_ticket: Array = ticket_payload[3]
	var encoded_auth_ticket = "".join(auth_ticket.map(func (b): return "%02X" % b))

	var request_result: Result = await HTTPUtils.fetch(
		Program.AUTH_SERVER_URI + AUTH_SERVER_SIGN_IN_PATH,
		["Content-Type: text/plain"],
		HTTPClient.METHOD_POST,
		encoded_auth_ticket,
	).settled
	
	if not request_result.is_ok():
		return request_result
	
	var response = request_result.unwrap()
	if response.response_code != HTTPClient.RESPONSE_OK:
		return Result.Err("failed to sign in: %s" % response.response_code)
	
	var body_text: String = response.body.get_string_from_utf8()
	return Result.Ok(JSON.parse_string(body_text))
