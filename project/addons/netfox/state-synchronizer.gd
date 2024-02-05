extends Node
class_name StateSynchronizer

# TODO: Custom icon

## Synchronizes state from authority.

@export var root: Node
@export var properties: Array[String]

var _property_cache: PropertyCache
var _props: Array[PropertyEntry]

var _last_received_tick: int = -1
var _last_received_state: Dictionary = {}

var network_aware := false


## Process settings.
##
## Call this after any change to configuration.
func process_settings() -> void:
	_property_cache = PropertyCache.new(root)
	_props = []

	for property in properties:
		var pe = _property_cache.get_entry(property)
		_props.push_back(pe)
	
	network_aware = root.has_method("_network_process")
	if network_aware:
		NetworkTime.on_tick.connect(_on_tick)
	else:
		NetworkTime.on_tick.disconnect(_on_tick)


func _ready() -> void:
	process_settings()
	NetworkTime.after_tick.connect(_after_tick)


func _on_tick(delta: float, tick: int) -> void:
	root._network_process(delta, tick)


func _after_tick(_delta: float, tick: int) -> void:
	if is_multiplayer_authority():
		# Submit snapshot
		var state = PropertySnapshot.extract(_props)
		_submit_state.rpc(state, tick)
	else:
		# Apply last received state
		PropertySnapshot.apply(_last_received_state, _property_cache)


@rpc("authority", "unreliable", "call_remote")
func _submit_state(state: Dictionary, tick: int) -> void:
	if tick <= _last_received_tick:
		return
		
	_last_received_state = state
	_last_received_tick = tick