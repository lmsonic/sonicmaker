extends Node2D
@onready var sensor: Sensor = $Sensor
@export var height_radius := 14.0
@export var speed := 1.0
const EXPLOSION = preload("res://enemies/explosion.tscn")
var facing_right := false:
	set(value):
		facing_right = value
		main_sprite.position.x = -3.0 if facing_right else 3.0
		main_sprite.flip_h = facing_right
		exhaust.position.x = -23.0 if facing_right else 23.0
		exhaust.flip_h = facing_right

var turning := false
@onready var main_sprite: AnimatedSprite2D = $MainSprite
@onready var exhaust: AnimatedSprite2D = $Exhaust

func _physics_process(_delta: float) -> void:
	if turning:
		move(0.0)
		turning = false
		return

	var result: Variant = sensor.sense_godot()
	if result:
		var distance: float = result.distance
		if distance < -8.0 or distance > 12.0:
			turn()
			move(distance)
		else:
			move(distance)

	else:
		turn()

func _on_attackable_attacked() -> void:
	var explosion: Node2D = EXPLOSION.instantiate()
	explosion.global_position = global_position
	get_tree().current_scene.add_child(explosion)
	queue_free()

func turn() -> void:
	facing_right = !facing_right
	turning = true


func move(floor_distance: float) -> void:
	var direction := + 1.0 if facing_right else -1.0
	global_position.x += direction
	global_position.y += floor_distance
