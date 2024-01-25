extends Node
class_name Authentication

var user_id: String
var user_name: String


func initialize() -> Result:
	var err := Result.Err("`initialize` not yet implemented")
	push_error(err.unwrap_err())
	return err
