@tool
class_name Spring extends SolidObject

enum Direction {
	Right,
	Up,
	Left,
	Down
}


@export var direction := Direction.Up:
	set(value):
		direction = value
		sprite.rotation = rotation_from_direction()
		if is_horizontal():
			width_radius = 8
			height_radius = 14
			sprite.position.x = -8.0 if direction == Direction.Left else 8.0
			sprite.position.y = 0.0
		else:
			width_radius = 16
			height_radius = 8
			sprite.position.x = 0.0
			sprite.position.y = -8.0 if direction == Direction.Up else 8.0

@export var spring_force := 10.0
@export var sprite: AnimatedSprite2D
func is_horizontal() -> bool:
	return direction == Direction.Right or direction == Direction.Left

func rotation_from_direction() -> float:
	match direction:
		Direction.Up: return 0.0
		Direction.Down: return PI
		Direction.Left: return -PI * 0.5
		Direction.Right: return PI * 0.5
	return 0.0

func _on_collided(collision: String, player: Character) -> void:
	if collision_matches_direction(collision):
		if is_horizontal():
			horizontal_spring(player)
		else:
			vertical_spring(player)
	elif is_horizontal() and player_not_moving_towards_spring(player) and check_box_around_player(player):
		horizontal_spring(player)

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

func direction_vector() -> Vector2:
	match direction:
		Direction.Up: return Vector2.UP
		Direction.Down: return Vector2.DOWN
		Direction.Left: return Vector2.LEFT
		Direction.Right: return Vector2.RIGHT
	return Vector2.ZERO



func vertical_spring(player: Character) -> void:
	if direction == Direction.Up:
		player.clear_standing_objects()
		player.has_jumped = false
		player.set_state("SpringBounce")
		player.spring_bounce_timer = 48


	player.velocity.y = -spring_force if direction == Direction.Up else spring_force
	player.global_position.y += 8 if direction == Direction.Up else -8
	sprite.play("spring")


func horizontal_spring(player: Character) -> void:
	if player.is_grounded:
		player.ground_speed = spring_force if direction == Direction.Right else -spring_force
	else:
		player.velocity.x = spring_force if direction == Direction.Right else -spring_force
		player.velocity.y = 0.0
	player.set_flip_h(direction == Direction.Left)
	player.control_lock_timer = 16
	sprite.play("spring")

func check_box_around_player(player: Character) -> bool:
	var contains_y := global_position.y - 24.0 <= player.global_position.y and player.global_position.y < global_position.y + 24.0
	var contains_x_right := global_position.x <= player.global_position.x and player.global_position.x <= global_position.x + 40.0
	var contains_x_left := global_position.x - 40.0 <= player.global_position.x and player.global_position.x <= global_position.x
	var contains_x := contains_x_right if direction == Direction.Right else contains_x_left
	return contains_y and contains_x

func player_not_moving_towards_spring(player: Character) -> bool:
	if player.velocity.x == 0.0:
		return true

	var delta := player.global_position - global_position
	# Moving to the right and player to the right
	if player.velocity.x > 0.0 and delta.x > 0.0:
		return true
	# Moving to the left and player to the left
	if player.velocity.x < 0.0 and delta.x < 0.0:
		return true
	return false

func _on_animated_sprite_2d_animation_finished() -> void:
	if sprite.animation.begins_with("spring"):
		sprite.play("relaxed")
