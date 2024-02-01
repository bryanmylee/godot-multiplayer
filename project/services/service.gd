extends Node
class_name Service


## [codeblock]
## @returns Result<null, int>
## [/codeblock]
func initialize() -> Result:
	return await Result.Err("`initialize` not yet implemented").to_promise().settled
