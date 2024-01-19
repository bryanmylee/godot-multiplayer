class_name Logger

static func client_log(args: Array, tags: Array = []):
	var id_str := "client(authority)(" + str(Program.client.peer_id) + "): " \
		if Program.is_server and Program.client.peer_id != 0 \
		else "client(" + str(Program.client.peer_id) + "): "
	
	var log_str := id_str + "".join(args)
	
	if not tags.is_empty():
		var tag_str := "[" + ",".join(tags) + "]"
		log_str = tag_str + " " + log_str
	
	print(log_str)
