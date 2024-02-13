extends Node
class_name AuthProvider

var id: String
var order: int
var provider_type: String
var provider_id: String
## [codeblock]
## Option<String>
## [/codeblock]
var email := Option.None()
var email_verified := false
## [codeblock]
## Option<String>
## [/codeblock]
var display_name := Option.None()
## [codeblock]
## Option<String>
## [/codeblock]
var user_name := Option.None()
## [codeblock]
## Option<String>
## [/codeblock]
var picture_url := Option.None()
## [codeblock]
## Option<String>
## [/codeblock]
var locale := Option.None()


## [codeblock]
## @returns Promise<null, String>
## [/codeblock]
func initialize() -> Result:
	var err := Result.Err("`initialize` not implemented")
	push_error(err.unwrap_err())
	return await err.to_promise().settled


## [codeblock]
## User {
##   id: String
##   name?: String
## }
##
## Token {
##   value: String
##   expires_at: String
## }
##
## UserWithAuthProviders {
##   user: User
##   providers: Array<AuthProvider>
## }
##
## SignInSuccess {
##   type: "success"
##   payload: {
##     access_token: Token
##     refresh_token: Token
##     user: UserWithAuthProviders
##   }
## }
##
## SignInPendingLinkOrCreate {
##   type: "pending_link_or_create"
##   payload: Array<UserWithAuthProviders>
## }
##
## SignInResult = SignInSuccess | SignInPendingLinkOrCreate
## [/codeblock]


## Sign in to the authentication server. The server will set a `access_token`
## cookie and also return the token as JSON.
## [codeblock]
## @returns Result<SignInResult, String>
## [/codeblock]
func server_sign_in() -> Result:
	var err := Result.Err("`server_sign_in` not implemented")
	push_error(err.unwrap_err())
	return await err.to_promise().settled
