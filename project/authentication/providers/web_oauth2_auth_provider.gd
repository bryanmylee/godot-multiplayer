extends AuthProvider
class_name WebOAuth2AuthProvider

@onready var oauth: WebOAuth2Service = ServiceManager.get_service("OAuth2")


func _init() -> void:
	name = "OAuth2"


func initialize() -> Result:
	oauth.consume_access_token_from_hash()
	var token_result := oauth.get_local_access_token()
	if token_result.is_none() or oauth.is_local_access_token_expired():
		oauth.reload_access_token_into_hash()
	
	var user_info_result := await oauth.get_user_info()
	if user_info_result.is_err():
		return user_info_result
	var user_info = user_info_result.unwrap()

	provider_type = "oauth2"
	provider_id = Option.new(user_info.id)
	email = Option.new(user_info.email)
	email_verified = user_info.verified_email
	user_name = Option.new(user_info.name)
	picture_url = Option.new(user_info.picture)
	locale = Option.new(user_info.locale)
	print("Logged in via the web as ", user_name.unwrap())
	
	return Result.Ok(null)


const AUTH_SERVER_SIGN_IN_PATH := "/auth/oauth2/sign-in"
func server_sign_in() -> Result:
	var access_token_result := oauth.get_local_access_token()
	if access_token_result.is_none():
		return Result.Err("access_token not loaded")
	var access_token = access_token_result.unwrap()
	
	var request_result: Result = await HTTPUtils.fetch(
		Env.AUTH_SERVER_URI + AUTH_SERVER_SIGN_IN_PATH,
		["Authorization: Bearer %s" % access_token],
		HTTPClient.METHOD_POST,
	).settled
	
	if request_result.is_err():
		return request_result
	
	var response = request_result.unwrap()
	if response.response_code != HTTPClient.RESPONSE_OK:
		return Result.Err("failed to sign in: %s" % response.response_code)
	
	var body_text: String = response.body.get_string_from_utf8()
	return Result.Ok(JSON.parse_string(body_text))
