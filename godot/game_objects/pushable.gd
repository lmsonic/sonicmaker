## From https://info.sonicretro.org/SPG:Solid_Objects#Pushable_Blocks and
##  https://info.sonicretro.org/SPG:Game_Objects#Pushable_Blocks
extends SolidObject

@export var fall_gravity := 0.21875
@export var push_speed := 1.0
@export var slide_off_speed := 4.0

@onready var sensor: Sensor = $Sensor
var is_falling := false
var velocity_y := 0.0
var pixels_moved := 0.0

func _physics_process(delta: float) -> void:
	physics_process(delta)
	var distance := sense_distance()
	if distance > 0:
		is_falling = true
	else:
		global_position.y += distance
		is_falling = false
		velocity_y = 0.0

	if is_falling:
		if absf(pixels_moved) > 16.0 || pixels_moved == 0.0:
			pixels_moved = 0.0
			distance = sense_distance()
			velocity_y += fall_gravity
			global_position.y += velocity_y
			if distance <= 0:
				is_falling = false
				global_position.y += distance
				velocity_y = 0.0
		elif pixels_moved != 0.0:
			var d := slide_off_speed if pixels_moved > 0.0 else -slide_off_speed
			pixels_moved += d
			global_position.x += d

# is on floor only when distance <=0
func sense_distance() -> float:
	var result: Variant = sensor.sense_godot()
	if result:
		return result.distance
	else:
		return 1.0


func _on_collided(collision: String, player: Character) -> void:
	match collision:
		"Left":
			global_position.x += push_speed
			player.global_position.x -= 1.0
			player.global_position.x += push_speed
			player.velocity.x = 0.0
			player.ground_speed = 0.25
			var distance := sense_distance()
			if distance > 0.0:
				player.set_state("Idle")
				pixels_moved = slide_off_speed
				global_position.x += slide_off_speed
				is_falling = true
		"Right":
			global_position.x -= push_speed
			player.global_position.x += 1.0
			player.global_position.x -= push_speed
			player.velocity.x = 0.0
			player.ground_speed = -0.25
			var distance := sense_distance()
			if distance > 0.0:
				player.set_state("Idle")
				pixels_moved = -slide_off_speed
				global_position.x -= slide_off_speed
				is_falling = true
