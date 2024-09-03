extends SolidObject
@export var fall_gravity := 0.21875
@export var rings := 10
@onready var sprite_2d: Sprite2D = $Sprite2D

@onready var sensor: Sensor = $Sensor
var destroyed:=false
var is_falling := false
var velocity_y := 0.0
func _physics_process(delta: float) -> void:
	if destroyed:
		return
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
const EXPLOSION = preload("res://enemies/explosion.tscn")
const DESTROYED_MONITOR = preload("res://game_objects/destroyed_monitor.tscn")

func _on_item_monitor_hitbox_area_entered(area: Area2D) -> void:
	if destroyed:
		return
	var hitbox := area as PlayerHitbox
	if hitbox and hitbox.player.attacking:
		if hitbox.player.velocity.y > 0.0:
			hitbox.player.velocity.y *= -1.0
		hitbox.increment_rings(rings)
		spawn(EXPLOSION)
		spawn(DESTROYED_MONITOR)
		queue_free()


func spawn(scene:PackedScene) -> Node2D:
	var node :Node2D= scene.instantiate()
	node.global_position = global_position
	get_tree().current_scene.add_child(node)
	return node
