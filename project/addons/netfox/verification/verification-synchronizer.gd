extends Node
class_name VerificationSynchronizer

@export var root: Node = get_parent()
## [codeblock]
## Array<keyof TState>
## [/codeblock]
@export var state_properties: Array[String]

@export var input_root: Node
## [codeblock]
## Array<keyof TInput>
## [/codeblock]
@export var input_properties: Array[String]

@onready var is_server := multiplayer.is_server()

var _record_state_props: Array[PropertyEntry] = []
var _record_input_props: Array[PropertyEntry] = []
var _auth_state_props: Array[PropertyEntry] = []
var _auth_input_props: Array[PropertyEntry] = []
var _nodes: Array[Node] = []

var _property_cache: PropertyCache
var _input_property_cache: PropertyCache

## [codeblock]
## Record<keyof TState, (TState[keyof TState], TState[keyof TState]) -> bool>
## [/codeblock]
var _verifiers := {}

## [codeblock]
## Dict<Tick, TState>
## [/codeblock]
var _states := {}
## [codeblock]
## Dict<Tick, TInput>
## [/codeblock]
var _inputs := {}

var _latest_confirmed_state_tick := -1
var _latest_received_input_tick := -1
var _latest_confirmed_input_tick = -1

static var _logger: _NetfoxLogger = _NetfoxLogger.for_netfox("VerificationSynchronizer")

## Process settings.
##
## Call this after any change to configuration.
func process_settings() -> void:
	_property_cache = PropertyCache.new(root)
	_input_property_cache = PropertyCache.new(input_root)

	_states.clear()
	_inputs.clear()

	_latest_confirmed_state_tick = -1
	_latest_received_input_tick = -1
	_latest_confirmed_input_tick = -1

	_record_state_props.clear()
	_auth_state_props.clear()
	for property in state_properties:
		var pe = _property_cache.get_entry(property)
		_record_state_props.push_back(pe)
		if pe.node.is_multiplayer_authority():
			_auth_state_props.push_back(pe)

	_record_input_props.clear()
	_auth_input_props.clear()
	for property in input_properties:
		var pe = _input_property_cache.get_entry(property)
		if pe.node.is_multiplayer_authority():
			_record_input_props.push_back(pe)
			_auth_input_props.push_back(pe)

	_verifiers.clear()
	for property in state_properties:
		var pe = _property_cache.get_entry(property)
		_verifiers[property] = Verifiers.find_for(pe.get_value())

	_nodes.clear()
	# Gather all verification-aware nodes to check during verification
	_nodes = root.find_children("*")
	_nodes.push_front(root)
	_nodes = _nodes.filter(func(it): return VerificationSynchronizer.is_verification_aware(it))
	_nodes.erase(self)


static func is_verification_aware(obj: Object) -> bool:
	return obj.has_method("_verified_tick")


func _ready() -> void:
	process_settings()

	NetworkTime.before_tick.connect(_before_tick)
	NetworkTime.on_tick.connect(_run_tick)
	NetworkTime.after_tick.connect(_after_tick)


func _before_tick(delta: float, tick: int) -> void:
	if input_root.is_multiplayer_authority():
		# Record local input
		var input := PropertySnapshot.extract(_record_input_props)
		_inputs[tick] = input
	
		# Resimulate ticks locally from `_latest_confirmed_state_tick` to prepare for `tick`.
		if _latest_confirmed_state_tick < tick - 1:
			NetworkTime.before_tick_loop.emit()
			for resim_tick in range(_latest_confirmed_state_tick, tick):
				_run_tick(delta, resim_tick)
				var resimulated_state := PropertySnapshot.extract(_record_state_props)
				_states[resim_tick + 1] = resimulated_state
			NetworkTime.after_tick_loop.emit()
	
	# elif is_server:
	# 	# Resimulate ticks server-side from `_latest_received_input_tick` to prepare for `tick`.
	# 	if _latest_received_input_tick < tick - 1:
	# 		print("server-side resimulation for ", range(_latest_received_input_tick, tick))
	# 		NetworkTime.before_tick_loop.emit()
	# 		for resim_tick in range(_latest_received_input_tick, tick):
	# 			_run_tick(delta, resim_tick)
	# 			var resimulated_state := PropertySnapshot.extract(_record_state_props)
	# 			_states[resim_tick + 1] = resimulated_state
	# 		NetworkTime.after_tick_loop.emit()


func _run_tick(delta: float, tick: int) -> void:
	# Set state to tick `t`
	var state := _get_latest_until_tick(_states, tick)
	PropertySnapshot.apply(state, _property_cache)
	# Set input to tick `t`
	var input := _get_latest_until_tick(_inputs, tick)
	PropertySnapshot.apply(input, _input_property_cache)

	# Applying input `t` to state `t`
	for node in _nodes:
		node._verified_tick(delta, tick)


func _after_tick(_delta: float, tick: int) -> void:
	var next_tick := tick + 1
	var next_state := PropertySnapshot.extract(_record_state_props)

	if not _record_state_props.is_empty() and next_tick > _latest_confirmed_state_tick:
		_states[next_tick] = next_state
	
	if is_server:
		_latest_confirmed_state_tick = next_tick
		var input_owner_id := input_root.get_multiplayer_authority()
		var next_input_tick = _latest_received_input_tick + 1
		# Broadcast state to controlling player
		if _latest_confirmed_input_tick < next_input_tick:
			_submit_confirmed_state.rpc_id(input_owner_id, next_state, next_input_tick)
			_latest_confirmed_input_tick = next_input_tick
		# Broadcast state to other players
		RPCUtils.rpc_except_id(input_owner_id, _submit_confirmed_state, next_state, next_tick)
	
	elif input_root.is_multiplayer_authority():
		# Send current input from local player to server
		_submit_input.rpc(_inputs[tick], tick)
	
	_trim_history()


func _trim_history() -> void:
	while _states.size() > NetworkRollback.history_limit:
		_states.erase(_states.keys().min())
	while _inputs.size() > NetworkRollback.history_limit:
		_inputs.erase(_inputs.keys().min())


func _get_latest_until_tick(buffer: Dictionary, tick: int) -> Dictionary:
	if buffer.has(tick):
		return buffer[tick]

	if buffer.is_empty():
		return {}
	
	var earliest = buffer.keys().min()
	var latest = buffer.keys().max()

	if tick < earliest:
		return buffer[earliest]
	
	if tick > latest:
		return buffer[latest]
	
	var before = buffer.keys() \
		.filter(func (key): return key < tick) \
		.max()
	
	return buffer[before]


@rpc("any_peer", "reliable", "call_remote")
func _submit_input(input: Dictionary, tick: int) -> void:
	var sender_id = multiplayer.get_remote_sender_id()
	var input_owner_id = input_root.get_multiplayer_authority()
	if input_owner_id != sender_id:
		_logger.warning("Received input for node owned by %s from %s, sender has no authority!" \
			% [input_owner_id, sender_id])
		return
	
	# Set input for tick.
	# Server-side resimulation of states from `_latest_received_input_tick` to
	# `NetworkTime.tick` in `before_tick`.
	_inputs[tick] = input
	_latest_received_input_tick = max(_latest_received_input_tick, tick)


@rpc("unreliable_ordered", "call_remote")
func _submit_confirmed_state(state: Dictionary, tick: int) -> void:
	if tick < NetworkTime.tick - NetworkRollback.history_limit and _latest_confirmed_state_tick >= 0:
		_logger.debug("Ignoring state %s older than %s frames" % [tick, NetworkRollback.history_limit])
		return
	
	if state.is_empty():
		_logger.warning("Received invalid state for tick %s" % [tick])
		return
	
	if not input_root.is_multiplayer_authority():
		_latest_confirmed_state_tick = NetworkTime.tick
		_states[NetworkTime.tick] = PropertySnapshot.merge(_states.get(NetworkTime.tick, {}), state)
		return
	
	if tick <= _latest_confirmed_state_tick:
		_logger.warning("Already received confirmed state for tick %s" % [tick])
		return
	
	# Set confirmed server state on local.
	# Local resimulation of states from `_latest_confirmed_state_tick` to
	# `NetworkTime.tick` in `before_tick`.
	_states[tick] = state
	_latest_confirmed_state_tick = max(_latest_confirmed_state_tick, tick)
