extends AuthProvider
class_name AppleGameCenterAuthProvider

@onready var game_center: AppleGameCenterService = ServiceManager.get_service("AppleGameCenter")


func _init() -> void:
	name = "AppleGameCenter"


func initialize() -> Result:
	var auth_result: Result = await game_center.authenticate().settled
	if auth_result.is_err():
		return auth_result
	var auth_data = auth_result.unwrap()

	provider_type = "game_center"
	provider_id = Option.new(auth_data.player_id)
	user_name = Option.new(auth_data.displayName)
	print("Logged in with Game Center as ", user_name.unwrap())

	return Result.Ok(null)


const AUTH_SERVER_SIGN_IN_PATH := "/auth/game-center/sign-in"
func server_sign_in() -> Result:
	if provider_id.is_none():
		return Result.Err("not authenticated locally")

	var id_signature_result: Result = await game_center.request_identity_verification_signature().settled
	if id_signature_result.is_err():
		return Result.Err("failed to get id signature from game center: %s" % id_signature_result.unwrap_err())
	
	var id_signature = id_signature_result.unwrap()
	
	var request_result: Result = await HTTPUtils.fetch(
		Program.AUTH_SERVER_URI + AUTH_SERVER_SIGN_IN_PATH,
		["Content-Type: application/json"],
		HTTPClient.METHOD_POST,
		JSON.stringify({
			"public_key_url": id_signature.public_key_url,
			"signature": id_signature.signature,
			"salt": id_signature.salt,
			"timestamp": id_signature.timestamp,
			"player_id": id_signature.player_id,
			"user_name": user_name.unwrap_or(null),
			"bundle_id": Program.IOS_BUNDLE_ID,
		}),
	).settled
	
	if request_result.is_err():
		return request_result
	
	var response = request_result.unwrap()
	if response.response_code != HTTPClient.RESPONSE_OK:
		return Result.Err("failed to sign in: %s" % response.response_code)
	
	var body_text: String = response.body.get_string_from_utf8()
	return Result.Ok(JSON.parse_string(body_text))
