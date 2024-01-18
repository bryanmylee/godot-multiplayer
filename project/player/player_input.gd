extends Node
class_name PlayerInput

@export_group("Dependencies")
@export var id_provider: IdentityProvider


@export_group("Synchronized State")
@export var is_crouching := false
@export var is_running := false
@export var direction := Vector2.ZERO


func _enter_tree() -> void:
	if Program.is_server or id_provider.is_local_player:
		set_multiplayer_authority(id_provider.id)


#region Predictive Events

#region just_jumped
var just_jumped := false
func jump() -> void:
	print("client(", Program.client.peer_id, "): player(", id_provider.id, "): just jumped")
	just_jumped = true
	__authority_jump.rpc_id(Program.game_authority_id, just_jumped)


@rpc("reliable")
func __authority_jump(_just_jumped: bool) -> void:
	print(
		"client(", Program.client.peer_id,
		"): player(", id_provider.id,
		"): just_jumped changed: ", _just_jumped
	)
	just_jumped = _just_jumped
	GameNetwork.rpc_clients_except_id(id_provider.id, __broadcast_jump, _just_jumped)


@rpc("reliable")
func __broadcast_jump(_just_jumped: bool) -> void:
	print(
		"client(", Program.client.peer_id,
		"): player(", id_provider.id,
		"): just_jumped changed: ", _just_jumped
	)
	just_jumped = _just_jumped
#endregion

#endregion


func _ready() -> void:
	set_process(id_provider.is_local_player)


func _process(_delta: float) -> void:
	is_crouching = Input.is_action_pressed("mod_crouch")
	is_running = Input.is_action_pressed("mod_run")
	direction = Input.get_vector("move_left", "move_right", "move_forward", "move_backward")
	if Input.is_action_just_pressed("jump"):
		jump()
