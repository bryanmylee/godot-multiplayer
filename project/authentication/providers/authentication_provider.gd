extends Node
class_name AuthenticationProvider

var user_id: String
var user_name: String

## [codeblock]
## @returns Promise<null, String>
## [/codeblock]
func initialize() -> Result:
	var err := Result.Err("`initialize` not implemented")
	push_error(err.unwrap_err())
	return await err.to_promise().settled


## [codeblock]
## User {
##   email?: String
##   email_verified: bool
##   locale?: String
##   oauth2_id?: String
##   oauth2_name?: String
##   oauth2_picture_url?: String
## }
## [/codeblock]


## Sign in to the authentication server. The server will set a `server_token`
## cookie and also return the token as JSON.
## [codeblock]
## @returns SignInResult {
##   server_token: String
##   user: User
## }
## [/codeblock]
func server_sign_in() -> Result:
	var err := Result.Err("`server_sign_in` not implemented")
	push_error(err.unwrap_err())
	return await err.to_promise().settled
