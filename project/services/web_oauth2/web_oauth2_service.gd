extends Service
class_name WebOAuth2Service

#region OAuth client info
const PROJECT_ID := "bryanmylee-multiplayer-base"
const AUTH_URI := "https://accounts.google.com/o/oauth2/v2/auth"
const CLIENT_ID := "865539732998-fibei7810dqa7ffbo5i65kevli7rfuef.apps.googleusercontent.com"

const REQUIRED_SCOPES := [
	"https://www.googleapis.com/auth/userinfo.email",
	"https://www.googleapis.com/auth/userinfo.profile",
	# "https://www.googleapis.com/auth/youtube.readonly",
]
#endregion


func _init() -> void:
	name = "OAuth2"


## Reloads the web client with the OAuth2.0 access token in the location hash `access_token`.
func reload_access_token_into_hash() -> void:
	if not OS.has_feature("web"):
		print("The JavaScriptBridge singleton is not available")
		return

	var current_uri = JavaScriptBridge.eval("window.location.origin + window.location.pathname;")
	
	var query_params = [
		"include_granted_scopes=true",
		"response_type=token",
		"scope=%s" % " ".join(REQUIRED_SCOPES),
		"redirect_uri=%s" % current_uri,
		"client_id=%s" % CLIENT_ID,
	]

	var request_url = AUTH_URI + "?" + "&".join(query_params)
	
	JavaScriptBridge.eval("window.open('%s', '_blank').focus(); window.close();" % request_url)


## Consume the access token received on the location hash into local storage.
func consume_access_token_from_hash() -> void:
	if not OS.has_feature("web"):
		print("The JavaScriptBridge singleton is not available")
		return
	
	JavaScriptBridge.eval("""
	const params = new URLSearchParams(window.location.hash.slice(1));
	if (params.has('access_token')) {
		window.localStorage.setItem('access_token', params.get('access_token'));
		params.delete('access_token');
		if (params.has('expires_in')) {
			const expiryDurationSeconds = params.get('expires_in');
			const expiresAtMs = +(new Date()) + expiryDurationSeconds * 1000;
			window.localStorage.setItem('access_token_expires_at', expiresAtMs);
			params.delete('expires_in');
		}
		params.delete('token_type');
		params.delete('scope');
		params.delete('authuser');
		params.delete('prompt');
	}
	window.history.replaceState(undefined, undefined, '#' + params.toString());
	""")


## [codeblock]
## @returns Option<String>
## [/codeblock]
func get_local_access_token() -> Option:
	if not OS.has_feature("web"):
		print("The JavaScriptBridge singleton is not available")
		return Option.None()
	
	var token = JavaScriptBridge.eval("window.localStorage.getItem('access_token');")
	if token == null:
		return Option.None()
	return Option.Some(token)


const USER_INFO_REQUEST_URI := "https://www.googleapis.com/userinfo/v2/me"
## The amount of data returned depends on the `userinfo.*` scopes granted above.
## [codeblock]
## UserInfo {
##   id: String
##   email: String
##   verified_email: bool
##   name: String
##   given_name: String
##   family_name: String
##   picture: String
##   locale: String
## }
##
## @returns Result<UserInfo, String>
## [/codeblock]
func get_user_info() -> Result:
	var access_token_result := get_local_access_token()
	if access_token_result.is_none():
		return Result.Err("access_token not loaded")
	var access_token = access_token_result.unwrap()
	
	var request_result: Result = await HTTPUtils.fetch(
		USER_INFO_REQUEST_URI, ["Authorization: Bearer %s" % access_token]
	).settled
	
	if request_result.is_err():
		push_error(request_result.unwrap_err())
		return request_result
	
	var response = request_result.unwrap()
	if response.response_code != HTTPClient.RESPONSE_OK:
		return Result.Err("failed to get data from Google's user info endpoint: %s" % response.response_code)
	
	var body_text: String = response.body.get_string_from_utf8()
	return Result.Ok(JSON.parse_string(body_text))
