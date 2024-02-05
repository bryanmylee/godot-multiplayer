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

## [codeblock]
## The amount of data returned depends on the `userinfo.*` scopes granted above.
## @returns UserInfo
## [/codeblock]
const USER_INFO_REQUEST_URI := "https://www.googleapis.com/userinfo/v2/me"
#endregion

var redirect_address := "http://localhost:8060/tmp_js_export.html"


func _init() -> void:
	name = "OAuth2"


## Reloads the web client with the OAuth2.0 access token in the location hash `access_token`.
func reload_access_token_into_hash() -> void:
	if not OS.has_feature("web"):
		print("The JavaScriptBridge singleton is not available")
		return
	
	var query_params = [
		"include_granted_scopes=true",
		"response_type=token",
		"scope=%s" % " ".join(REQUIRED_SCOPES),
		"redirect_uri=%s" % redirect_address,
		"client_id=%s" % CLIENT_ID,
	]

	var auth_code_request_url = AUTH_URI + "?" + "&".join(query_params)
	
	JavaScriptBridge.eval("window.open('%s', '_blank').focus(); window.close();" % auth_code_request_url)


## Consume the access token received on the location hash into local storage.
func consume_access_token_from_hash() -> void:
	if not OS.has_feature("web"):
		print("The JavaScriptBridge singleton is not available")
		return
	
	JavaScriptBridge.eval("""
	const params = new URLSearchParams(window.location.hash.slice(1));
	if (params.has('access_token')) {
		window.localStorage.setItem('access_token', params.get('access_token'));
		if (params.has('expires_in')) {
			const expiryDurationSeconds = params.get('expires_in');
			const expiresAtMs = +(new Date()) + expiryDurationSeconds * 1000;
			window.localStorage.setItem('access_token_expires_at', expiresAtMs);
		}
		window.history.replaceState(undefined, undefined, '#');
	}
	""")


## [codeblock]
## @returns Option<String>
## [/codeblock]
func get_access_token() -> Option:
	if not OS.has_feature("web"):
		print("The JavaScriptBridge singleton is not available")
		return Option.None()
	
	var token = JavaScriptBridge.eval("window.localStorage.getItem('access_token');")
	if token == null:
		return Option.None()
	return Option.Some(token)


## [codeblock]
## @returns Result<UserInfo {
##   id: String
##   email: String
##   verified_email: bool
##   name: String
##   given_name: String
##   family_name: String
##   picture: String
##   locale: String
## }, String>
## [/codeblock]
func get_user_info() -> Result:
	var access_token_result := get_access_token()
	if access_token_result.is_none():
		return Result.Err("access_token not loaded")
	var access_token = access_token_result.unwrap()


	var http_request := HTTPRequest.new()
	http_request.accept_gzip = false
	add_child(http_request)

	var request_handler := Promise.new(func(resolve, reject):
		http_request.request_completed.connect(func (
			result: int,
			response_code: int,
			_headers: PackedStringArray,
			body: PackedByteArray,
		):
			http_request.queue_free()
			if result != HTTPRequest.RESULT_SUCCESS:
				return reject.call("unsuccessful http request: %s" % result)
			if response_code != HTTPClient.RESPONSE_OK:
				return reject.call("non-200 response code received: %s" % response_code)
			var body_text := body.get_string_from_utf8()
			var body_json = JSON.parse_string(body_text)
			return resolve.call(body_json)
		)
	)

	var request_result := Result.from_gderr(http_request.request(
		USER_INFO_REQUEST_URI, ["Authorization: Bearer %s" % access_token]
	))

	if request_result.is_err():
		push_error(request_result.unwrap_err())
		return request_result
	
	return await request_handler.settled
