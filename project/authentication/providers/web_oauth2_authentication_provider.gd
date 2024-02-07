extends AuthenticationProvider
class_name WebOAuth2AuthenticationProvider

@onready var oauth: WebOAuth2Service = ServiceManager.get_service("OAuth2")


func _init() -> void:
	name = "OAuth2"


func initialize() -> Result:
	oauth.consume_access_token_from_hash()
	var token_result := oauth.get_local_access_token()
	if token_result.is_none():
		oauth.reload_access_token_into_hash()
	
	var user_info_result := await oauth.get_user_info()
	if user_info_result.is_err():
		return user_info_result
	var user_info = user_info_result.unwrap()

	user_id = user_info.id
	user_name = user_info.name
	
	return Result.Ok(null)


const AUTH_SERVER_SIGN_IN_PATH := "/auth/oauth2/sign_in"
func server_sign_in() -> Result:
	var access_token_result := oauth.get_local_access_token()
	if access_token_result.is_none():
		return Result.Err("access_token not loaded")
	var access_token = access_token_result.unwrap()
	
	var request_result: Result = await HTTPUtils.fetch(
		Program.AUTH_SERVER_URI + AUTH_SERVER_SIGN_IN_PATH,
		["Authorization: Bearer %s" % access_token],
		HTTPClient.METHOD_POST,
	).settled
	
	if not request_result.is_ok():
		return request_result
	
	var response = request_result.unwrap()
	if response.response_code != HTTPClient.RESPONSE_OK:
		return Result.Err("failed to get data from Google's user info endpoint: %s" % response.response_code)
	
	var body_text: String = response.body.get_string_from_utf8()
	return Result.Ok(JSON.parse_string(body_text))
