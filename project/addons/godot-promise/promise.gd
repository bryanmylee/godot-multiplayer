extends RefCounted
class_name Promise


enum Status {
	RESOLVED,
	REJECTED
}


signal settled(result: Result)
signal resolved(value: Variant)
signal rejected(reason: Variant)


## Generic rejection reason
const PROMISE_REJECTED := "Promise rejected"


var is_settled := false


func _init(callable: Callable):
	resolved.connect(
		func(value: Variant): 
			is_settled = true
			settled.emit(Result.Ok(value)),
		CONNECT_ONE_SHOT
	)
	rejected.connect(
		func(rejection: Variant):
			is_settled = true
			settled.emit(Result.Err(rejection)),
		CONNECT_ONE_SHOT
	)
	
	callable.call_deferred(
		func(value: Variant):
			if not is_settled:
				resolved.emit(value),
		func(rejection: Variant):
			if not is_settled:
				rejected.emit(rejection)
	)


func then(resolved_callback: Callable) -> Promise:
	resolved.connect(
		resolved_callback, 
		CONNECT_ONE_SHOT
	)
	return self


func catch(rejected_callback: Callable) -> Promise:
	rejected.connect(
		rejected_callback, 
		CONNECT_ONE_SHOT
	)
	return self


static func from(input_signal: Signal) -> Promise:
	return Promise.new(
		func(resolve: Callable, _reject: Callable):
			var number_of_args := input_signal.get_object().get_signal_list() \
				.filter(func(signal_info: Dictionary) -> bool: return signal_info["name"] == input_signal.get_name()) \
				.map(func(signal_info: Dictionary) -> int: return signal_info["args"].size()) \
				.front() as int
			
			if number_of_args == 0:
				await input_signal
				resolve.call(null)
			else:
				# only one arg in signal is allowed for now
				var result = await input_signal
				resolve.call(result)
	)


static func from_many(input_signals: Array[Signal]) -> Array[Promise]:
	return input_signals.map(
		func(input_signal: Signal): 
			return Promise.from(input_signal)
	)

	
static func all(promises: Array[Promise]) -> Promise:
	return Promise.new(
		func(resolve: Callable, reject: Callable):
			if promises.is_empty():
				resolve.call(null)
			var resolved_promises: Array[bool] = []
			var results := []
			results.resize(promises.size())
			resolved_promises.resize(promises.size())
			resolved_promises.fill(false)
	
			for i in promises.size():
				promises[i].then(
					func(value: Variant):
						results[i] = value
						resolved_promises[i] = true
						if resolved_promises.all(func(_value: bool): return _value):
							resolve.call(results)
				).catch(
					func(rejection: Variant):
						reject.call(rejection)
				)
	)


static func any(promises: Array[Promise]) -> Promise:
	return Promise.new(
		func(resolve: Callable, reject: Callable):
			if promises.is_empty():
				reject.call("At least one promise is required")
			var rejected_promises: Array[bool] = []
			var rejections: Array[Variant] = []
			rejections.resize(promises.size())
			rejected_promises.resize(promises.size())
			rejected_promises.fill(false)
	
			for i in promises.size():
				promises[i].then(
					func(value: Variant): 
						resolve.call(value)
				).catch(
					func(rejection: Variant):
						rejections[i] = rejection
						rejected_promises[i] = true
						if rejected_promises.all(func(value: bool): return value):
							reject.call(rejections)
				)
	)
