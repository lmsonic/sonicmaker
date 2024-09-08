@tool class_name Spike extends SolidObject

@export var moving := false

enum Direction {
	Up = 0,
	Right = 1,
	Down = 2,
	Left = 3,
}

@export var direction := Direction.Up:
	set(value):
		direction = value
		if !sprite:
			return
		sprite.rotation = rotation_from_direction()

@export var sprite: Sprite2D
func is_horizontal() -> bool:
	return direction == Direction.Right or direction == Direction.Left

func direction_vector() -> Vector2:
	match direction:
		Direction.Up: return Vector2.UP
		Direction.Down: return Vector2.DOWN
		Direction.Left: return Vector2.LEFT
		Direction.Right: return Vector2.RIGHT
	return Vector2.ZERO

func rotation_from_direction() -> float:
	match direction:
		Direction.Up: return 0.0
		Direction.Down: return PI
		Direction.Left: return -PI * 0.5
		Direction.Right: return PI * 0.5
	return 0.0

func collision_matches_direction(collision: String) -> bool:
	match collision:
		"Left":
			return direction == Direction.Left
		"Right":
			return direction == Direction.Right
		"Up":
			return direction == Direction.Up
		"Down":
			return direction == Direction.Down
	return false

var timer := 0
var extended := false
@onready var original_position := global_position

func _physics_process(delta: float) -> void:
	if Engine.is_editor_hint():
		return
	physics_process(delta)
	if moving:
		timer += 1
		if timer >= 64:
			move()

func move() -> void:
	var dir := direction_vector()
	if extended:
		# Retracting
		position -= dir * 8.0
		if position.distance_squared_to(original_position) < 0.5:
			position = original_position
			timer = 0
			extended = false

	else:
		# Extending
		position += dir * 8.0
		if position.distance_squared_to(original_position + dir * 32.0) < 0.5:
			position = original_position + dir * 32.0
			timer = 0
			extended = true


func _on_collided(collision: String, player: Character) -> void:
	if collision_matches_direction(collision):
		player.on_hurt(self)
