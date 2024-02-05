extends Object
class_name Verifiers

class Verifier:
	var is_applicable: Callable
	var _internal_verify: Callable
	var default_margin: Variant


	func verify(a, b, margin = null) -> bool:
		return _internal_verify.call(a, b, default_margin if margin == null else margin)


	func _init(_is_applicable: Callable, _verify: Callable, _default_margin: Variant):
		is_applicable = _is_applicable
		_internal_verify = _verify
		default_margin = _default_margin


static func verify_default(a, b, _margin):
	return a == b


static var verifiers: Array[Verifier]
static var default_verify: Callable = func(a, b): return a == b


## Register a verifier.
##
## New verifiers are pushed to the front of the list, making them have 
## precedence over existing ones. This can be useful in case you want to override
## the built-in verifiers.
static func register(is_applicable: Callable, apply: Callable, default_margin: Variant):
	verifiers.push_front(Verifier.new(is_applicable, apply, default_margin))


## Find the appropriate verifier for the given value.
##
## If none was found, the default verifier is returned.
static func find_for(value) -> Callable:
	for verifier in verifiers:
		if verifier.is_applicable.call(value):
			return verifier.verify
	return verify_default


## Verify two values.
##
## Note, that it is usually faster to just cache the Callable returned by find_for
## and call that, instead of calling verify repeatedly. The latter will have 
## to lookup the appropriate verifier on every call.
static func verify(a, b, margin):
	return find_for(a).call(a, b, margin)


static func is_float(a): return a is float
static func verify_float(a: float, b: float, margin: float):
	return absf(a - b) < margin

static func is_vec2(a): return a is Vector2
static func verify_vec2(a: Vector2, b: Vector2, margin: Vector2):
	return absf(a.x - b.x) < margin.x and absf(a.y - b.y) < margin.y

static func is_vec3(a): return a is Vector3
static func verify_vec3(a: Vector3, b: Vector3, margin: Vector3):
	return absf(a.x - b.x) < margin.x and absf(a.y - b.y) < margin.y and absf(a.z - b.z) < margin.z

static func is_transform2(a): return a is Transform2D
static func verify_transform2(a: Transform2D, b: Transform2D, margin: Transform2D):
	return verify_vec2(a.origin, b.origin, margin.origin) \
		and verify_vec2(a.x, b.x, margin.x) \
		and verify_vec2(a.y, b.y, margin.y)

static func is_transform3(a): return a is Transform3D
static func verify_transform3(a: Transform3D, b: Transform3D, margin: Transform3D):
	return verify_vec3(a.origin, b.origin, margin.origin) \
		and verify_vec3(a.basis.x, b.basis.x, margin.basis.x) \
		and verify_vec3(a.basis.y, b.basis.y, margin.basis.y) \
		and verify_vec3(a.basis.z, b.basis.z, margin.basis.z)


static func _static_init():
	# Register built-in verifiers
	# Float
	Verifiers.register(Verifiers.is_float, Verifiers.verify_float, 0.1)
	# Vector
	Verifiers.register(Verifiers.is_vec2, Verifiers.verify_vec2, Vector2(0.1, 0.1))
	Verifiers.register(Verifiers.is_vec3, Verifiers.verify_vec3, Vector3(0.1, 0.1, 0.1))
	# Transform
	var default_transform2_margin := Transform2D()
	default_transform2_margin.x = Vector2(0.1, 0.1)
	default_transform2_margin.y = Vector2(0.1, 0.1)
	default_transform2_margin.origin = Vector2(0.1, 0.1)
	Verifiers.register(Verifiers.is_transform2, Verifiers.verify_transform2, default_transform2_margin)

	var default_transform3_margin := Transform3D()
	default_transform3_margin.basis.x = Vector3(0.1, 0.1, 0.1)
	default_transform3_margin.basis.y = Vector3(0.1, 0.1, 0.1)
	default_transform3_margin.basis.z = Vector3(0.1, 0.1, 0.1)
	default_transform3_margin.origin = Vector3(0.1, 0.1, 0.1)
	Verifiers.register(Verifiers.is_transform3, Verifiers.verify_transform3, default_transform3_margin)
