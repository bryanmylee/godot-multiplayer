extends Object
class_name Interpolators

class Interpolator:
	var is_applicable: Callable
	var apply: Callable
	
	func _init(_is_applicable: Callable, _apply: Callable):
		is_applicable = _is_applicable
		apply = _apply

static func interpolate_default(a, b, f: float):
	return a if f < 0.5 else b

static var interpolators: Array[Interpolator]

## Register an interpolator.
##
## New interpolators are pushed to the front of the list, making them have 
## precedence over existing ones. This can be useful in case you want to override
## the built-in interpolators.
static func register(is_applicable: Callable, apply: Callable):
	interpolators.push_front(Interpolator.new(is_applicable, apply))

## Find the appropriate interpolator for the given value.
##
## If none was found, the default interpolator is returned.
static func find_for(value) -> Callable:
	for interpolator in interpolators:
		if interpolator.is_applicable.call(value):
			return interpolator.apply
	
	return interpolate_default

## Interpolate between two values.
##
## Note, that it is usually faster to just cache the Callable returned by find_for
## and call that, instead of calling interpolate repeatedly. The latter will have 
## to lookup the appropriate interpolator on every call.
static func interpolate(a, b, f: float):
	return find_for(a).call(a, b, f)


static func is_float(a): return a is float
static func interpolate_float(a: float, b: float, f: float):
	return lerpf(a, b, f)

static func is_vec2(a): return a is Vector2
static func interpolate_vec2(a: Vector2, b: Vector2, f: float):
	return a.lerp(b, f)

static func is_vec3(a): return a is Vector3
static func interpolate_vec3(a: Vector3, b: Vector3, f: float):
	return a.lerp(b, f)

static func is_transform2(a): return a is Transform2D
static func interpolate_transform2(a: Transform2D, b: Transform2D, f: float):
	return a.interpolate_with(b, f)

static func is_transform3(a): return a is Transform3D
static func interpolate_transform3(a: Transform3D, b: Transform3D, f: float):
	return a.interpolate_with(b, f)


static func _static_init():
	# Register built-in interpolators
	# Float
	Interpolators.register(Interpolators.is_float, Interpolators.interpolate_float)
	# Vector
	Interpolators.register(Interpolators.is_vec2, Interpolators.interpolate_vec2)
	Interpolators.register(Interpolators.is_vec3, Interpolators.interpolate_vec3)
	# Transform
	Interpolators.register(Interpolators.is_transform2, Interpolators.interpolate_transform2)
	Interpolators.register(Interpolators.is_transform3, Interpolators.interpolate_transform3)
