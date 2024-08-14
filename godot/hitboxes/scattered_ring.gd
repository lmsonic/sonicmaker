extends Ring
@onready var sensor: Sensor = $Sensor

@export var ring_gravity := 0.09375
var velocity := Vector2.ZERO
var lifespan := 256.0



func _physics_process(delta: float) -> void:
	velocity.y += ring_gravity
	if velocity.y > 0:
		var distance := sense()
		if distance <= 0:
			# Touched the floor
			velocity.y *= -0.75
	global_position += velocity

	lifespan -= 1
	sprite.speed_scale = lifespan / 256.0
	if lifespan <= 0:
		queue_free()


func sense() -> float:
	var result:Variant = sensor.sense_godot()
	if result:
		return result.distance
	else:
		return 1.0
