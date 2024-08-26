extends SolidObject
@export var fall_gravity := 0.21875
@export var rings := 10

@onready var sensor: Sensor = $Sensor
var is_falling := false
var velocity_y := 0.0
func _physics_process(delta: float) -> void:
	physics_process(delta)
	var distance := sense_distance()
	if distance > 0:
		velocity_y += fall_gravity
	else:
		global_position.y += distance
		velocity_y = 0.0
	global_position.y += velocity_y

# is on floor only when distance <=0
func sense_distance() -> float:
	var result: Variant = sensor.sense_godot()
	if result:
		return result.distance
	else:
		return 1.0


func _on_item_monitor_hitbox_area_entered(area: Area2D) -> void:
	var hitbox := area as PlayerHitbox
	if hitbox and hitbox.player.attacking:
		if hitbox.player.velocity.y > 0.0:
			hitbox.player.velocity.y *= -1.0
		hitbox.increment_rings(rings)
		queue_free()
