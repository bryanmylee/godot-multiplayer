extends Service
class_name AppleGameCenterService

var game_center


func _init() -> void:
	name = "AppleGameCenter"


func initialize() -> Result:
	if not Engine.has_singleton("GameCenter"):
		set_process(false)
		return Result.Err("Game Center not available")
	
	game_center = Engine.get_singleton("GameCenter")
	print("Game Center initialized")

	return Result.Ok(null)


signal pending_event(payload: Variant)
func _process(_delta: float) -> void:
	if game_center.get_pending_event_count() > 0:
		var event = game_center.pop_pending_event()
		print("Game Center received event: ", event)
		pending_event.emit(event)


## [codeblock]
## AuthenticationSuccess {
##   type: "authentication"
##   result: "ok"
##   alias: String
##   displayName: String
##   player_id: String
## }
##
## AuthenticationError {
##   type: "authentication"
##   result: "error"
##   error_code: int
##   error_description: String
## }
##
## AuthenticationResult = |
##   | AuthenticationSuccess
##   | AuthenticatoinError
##
## @returns Promise<AuthenticationResult>
## [/codeblock]
func authenticate() -> Promise:
	game_center.authenticate()

	return Promise.new(func (resolve, reject):
		var response_handler := func (payload: Dictionary):
			if payload.type != "authentication":
				return
			if payload.result != "ok":
				reject.call(payload)
			else:
				resolve.call(payload)
		
		# Auto-disconnected when `response_handler` is deallocated.
		pending_event.connect(response_handler)
	)


## [codeblock]
## RequestIdVerificationSignatureSuccess {
##   type: "identity_verification_signature"
##   result: "ok"
##   public_key_url: String
##   signature: String
##   salt: String
##   timestamp: int
##   player_id: String
## }
##
## RequestIdVerificationSignatureError {
##   type: "identity_verification_signature"
##   result: "error"
##   error_code: int
##   error_description: String
## }
##
## RequestIdVerificationSignatureResult = |
##   | RequestIdVerificationSignatureSuccess
##   | RequestIdVerificationSignatureError
##
## @returns Promise<RequestIdVerificationSignatureResult>
## [/codeblock]
func request_identity_verification_signature():
	game_center.request_identity_verification_signature()

	return Promise.new(func (resolve, reject):
		var response_handler := func (payload: Dictionary):
			if payload.type != "identity_verification_signature":
				return
			if payload.result != "ok":
				reject.call(payload)
			else:
				resolve.call(payload)
		
		# Auto-disconnected when `response_handler` is deallocated.
		pending_event.connect(response_handler)
	)
