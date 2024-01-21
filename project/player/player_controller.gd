extends Synchronized
class_name PlayerController

func _enter_tree() -> void:
	if id_provider.is_local_player or Program.is_game_authority:
		set_multiplayer_authority(id_provider.id)
	else:
		set_multiplayer_authority(Program.game_authority_id)


#region Synchronized State
var is_crouching := false :
	set(new):
		if is_crouching == new: return
		is_crouching = new
		if id_provider.is_local_player:
			sync_to_authority("is_crouching", new)
		elif Program.is_game_authority:
			authority_sync_to_peers("is_crouching", new)

var is_running := false :
	set(new):
		if is_running == new: return
		is_running = new
		if id_provider.is_local_player:
			sync_to_authority("is_running", new)
		elif Program.is_game_authority:
			authority_sync_to_peers("is_running", new)

var direction := Vector2.ZERO :
	set(new):
		if direction == new: return
		direction = new
		if id_provider.is_local_player:
			sync_to_authority("direction", new)
		elif Program.is_game_authority:
			authority_sync_to_peers("direction", new)
#endregion


#region Synchronized Events
var just_jumped := false :
	set(new):
		if just_jumped == new: return
		just_jumped = new
		if id_provider.is_local_player:
			sync_reliable_to_authority("just_jumped", new)
		elif Program.is_game_authority:
			authority_sync_reliable_to_peers("just_jumped", new)
#endregion


func _ready() -> void:
	set_process(id_provider.is_local_player)


func _process(_delta: float) -> void:
	is_crouching = Input.is_action_pressed("mod_crouch")
	is_running = Input.is_action_pressed("mod_run")
	direction = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
	if Input.is_action_just_pressed("jump"):
		just_jumped = true
