class_name ObjectSerializer

static func to_dict(object: Object) -> Dictionary:
	var dict = {}
	for prop in object.get_property_list():
		if prop.name == "RefCounted" \
			or prop.name == "script" \
			or prop.name == "Built-in script":
			continue
		dict[prop.name] = to_dict(object[prop.name]) if object[prop.name] is Object else object[prop.name]
	return dict
