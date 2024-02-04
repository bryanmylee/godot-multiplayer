class_name Symbol
## `Symbol` allows us to define a globally unique value.

var _id: int
var description: String


func _init(_description := ""):
	_id = randi()
	description = _description


func is_equal(other: Variant) -> bool:
	return other is Symbol and _id == other._id
