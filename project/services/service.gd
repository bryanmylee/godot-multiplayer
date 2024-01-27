extends Node
class_name Service


func initialize() -> Result:
	return await Result.Err("`initialize` not yet implemented").to_promise().settled
