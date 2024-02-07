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
