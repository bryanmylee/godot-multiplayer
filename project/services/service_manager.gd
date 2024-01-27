extends Node

var services_node: Node

var services: Array[Service] :
	get:
		var _services: Array[Service] = []
		_services.assign(services_node.get_children())
		return _services


func get_service(sname: String) -> Service:
	return services_node.find_child(sname) as Service


func _ready():
	services_node = Node.new()
	services_node.name = "Services"
	add_child(services_node)

	initialize_default_services()


func initialize_default_services():
	match OS.get_name():
		"Windows", "macOS", "Linux", "FreeBSD", "NetBSD", "OpenBSD", "BSD":
			initialize_service("res://services/steam_service.gd")
		"Android":
			pass
		"iOS":
			initialize_service("res://services/apple_game_center_service.gd")
		"Web":
			pass


func initialize_service(service_path: String) -> Result:
	var service := load(service_path).new() as Service
	services_node.add_child(service)
	service.owner = services_node

	var init_result := await service.initialize()
	if init_result.is_err():
		return init_result
	
	return Result.Ok(null)
