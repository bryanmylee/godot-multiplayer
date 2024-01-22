extends NetworkSynchronized
class_name PlayerController

func _enter_tree() -> void:
	if id_provider.is_local_player or Program.is_game_authority:
		set_multiplayer_authority(id_provider.id)
	else:
		set_multiplayer_authority(Program.game_authority_id)


#region Synchronized State
var is_crouching := false
@rpc
func sync_is_crouching(new: bool) -> void:
	is_crouching = new
	if Program.is_game_authority:
		GameNetwork.rpc_clients_except_id(id_provider.id, sync_is_crouching, is_crouching)

var is_running := false
@rpc
func sync_is_running(new: bool) -> void:
	is_running = new
	if Program.is_game_authority:
		GameNetwork.rpc_clients_except_id(id_provider.id, sync_is_running, is_running)


var direction := Vector2.ZERO
@rpc
func sync_direction(new: Vector2) -> void:
	direction = new
	if Program.is_game_authority:
		GameNetwork.rpc_clients_except_id(id_provider.id, sync_direction, direction)
#endregion


#region Synchronized Events
var just_jumped := false :
	set(new):
		if just_jumped == new: return
		just_jumped = new
		if just_jumped:
			if id_provider.is_local_player:
				sync_reliable_to_authority("just_jumped", just_jumped)
			elif Program.is_game_authority:
				authority_sync_reliable_to_peers("just_jumped", just_jumped)
#endregion


func _ready() -> void:
	set_process(id_provider.is_local_player)
	set_synchronization_process(id_provider.is_local_player)


func _process(_delta: float) -> void:
	is_crouching = Input.is_action_pressed("mod_crouch")
	is_running = Input.is_action_pressed("mod_run")
	direction = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
	if Input.is_action_just_pressed("jump"):
		just_jumped = true


func _synchronization_process() -> void:
	sync_is_crouching.rpc_id(Program.game_authority_id, is_crouching)
	sync_is_running.rpc_id(Program.game_authority_id, is_running)
	sync_direction.rpc_id(Program.game_authority_id, direction)
