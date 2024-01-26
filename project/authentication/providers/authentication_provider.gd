extends Node
class_name AuthenticationProvider

var user_id: String
var user_name: String


func initialize() -> Result:
	var err := Result.Err("`initialize` not implemented")
	push_error(err.unwrap_err())
	return err
