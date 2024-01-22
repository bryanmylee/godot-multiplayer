extends RefCounted
class_name HistoryRecorder
"""
<TValue>
"""

var _head_idx := 0
var capacity := 0
"Array<[key: int, value: TValue]>"
var values: Array = []
"[key: int, value: TValue] | null"
var latest: Variant :
	get: return values[posmod(_head_idx - 1, capacity)]


func _init(_capacity := 50) -> void:
	capacity = _capacity
	values.resize(capacity)


## `max_key` must be larger than all other keys previously inserted.
func append(max_key: int, value: Variant) -> void:
	"""
	@param value: TValue
	"""
	values[_head_idx] = [max_key, value]
	_head_idx = (_head_idx + 1) % capacity


## Search for a matching history record. [br]
## [br]
## If `key` matches a record exactly, this returns that record.
## If `key` exists between two records, this returns both records.
## Otherwise, this returns `null`.
func search(key: int) -> Variant:
	"""
	@returns [[key: int, value: TValue]] | [[key: int, value: TValue], [key: int, value: TValue]] | null
	"""
	var curr_idx := _head_idx
	for _i in range(capacity):
		var curr = values[curr_idx]
		# no current.
		if curr == null:
			return null
		var curr_key: int = curr[0]
		# find the first entry larger than `key`.
		if curr_key == key:
			return [curr]
		elif curr_key >= key:
			var prev_idx := posmod(curr_idx - 1, capacity)
			var prev = values[prev_idx]
			# no previous.
			if prev == null:
				return null
			var prev_key: int = prev[0]
			# no valid previous.
			if prev_key >= key:
				return null
			return [prev, curr]
		curr_idx = posmod(curr_idx + 1, capacity)
	return null
