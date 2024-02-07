extends Node


## [codeblock]
## @returns Promise<Response {
##   result: HTTPRequest.Result
##   response_code: HTTPClient.ResponseCode
##   headers: PackedStringArray
##   body: PackedByteArray
## }, String>
## [/codeblock]
func fetch(
	url: String,
	req_headers: PackedStringArray = PackedStringArray(),
	method: HTTPClient.Method = HTTPClient.METHOD_GET,
	req_data: String = "",
) -> Promise:
	var http_request := HTTPRequest.new()
	if OS.has_feature("web"):
		## HTML5 exports fail to decompress gzip payloads.
		http_request.accept_gzip = false
	add_child(http_request)
	
	var request_handler := Promise.new(func(resolve, reject):
		http_request.request_completed.connect(func (
			result: HTTPRequest.Result,
			response_code: HTTPClient.ResponseCode,
			headers: PackedStringArray,
			body: PackedByteArray,
		):
			http_request.queue_free()
			if result != HTTPRequest.RESULT_SUCCESS:
				return reject.call("unsuccessful http request: %s" % result)
			return resolve.call({
				"result": result,
				"response_code": response_code,
				"headers": headers,
				"body": body,
			}),
			CONNECT_ONE_SHOT,
		)
	)
	
	var request_result := Result.from_gderr(http_request.request(
		url, req_headers, method, req_data
	))

	if request_result.is_err():
		push_error(request_result.unwrap_err())
		return request_result.to_promise()
	
	return request_handler
