## From https://info.sonicretro.org/SPG:Game_Objects#Diagonal_Springs
@tool extends SlopedSolidObject

enum Direction {
	TopRight,
	TopLeft,
	BottomRight,
	BottomLeft
}
@export var debug_trajectory := false:
	set(value):
		debug_trajectory = value
		queue_redraw()

func is_bottom(dir: Direction) -> bool:
	return dir == Direction.BottomRight || dir == Direction.BottomLeft

func is_top(dir: Direction) -> bool:
	return dir == Direction.TopRight || dir == Direction.TopLeft

func is_right(dir: Direction) -> bool:
	return dir == Direction.BottomRight || dir == Direction.TopRight

func is_left(dir: Direction) -> bool:
	return dir == Direction.TopLeft || dir == Direction.BottomLeft

func direction_vector() -> Vector2:
	match direction:
		Direction.TopRight: return Vector2(1.0, -1.0)
		Direction.TopLeft: return Vector2(-1.0, -1.0)
		Direction.BottomRight: return Vector2(1.0, 1.0)
		Direction.BottomLeft: return Vector2(-1.0, 1.0)
	return Vector2.ZERO


func _on_collided(collision: String, player: Character) -> void:
	match collision:
		"Up":
			if is_top(direction) and check_horizontal_conditions(player):
				spring(player)
		"Down":
			if is_bottom(direction) and check_horizontal_conditions(player):
				spring(player)


func _draw() -> void:
	if !debug_trajectory:

		return
	const g := Vector2(0.0, 0.21875)
	const top_speed = 6.0
	var initial_velocity := direction_vector() * spring_force
	initial_velocity.x = clampf(initial_velocity.x, -top_speed, top_speed)
	var time := 0.0
	var delta := 0.5
	var old_position := Vector2.ZERO
	for i in 40:
		time += delta
		var p := initial_velocity * time + 0.5 * g * time * time
		draw_dashed_line(old_position, p, Color.WHITE, 0.5)
		old_position = p

@export var direction := Direction.TopRight:
	set(value):
		if is_bottom(direction) and is_top(value) || is_bottom(value) and is_top(direction):
			flip_y()
		if is_left(direction) and is_right(value) || is_left(value) and is_right(direction):
			flip_x()
		direction = value
		update_sprite()
		queue_redraw()


func update_sprite() -> void:
	if is_right(direction):
		sprite.position.x = 10.0
	elif is_left(direction):
		sprite.position.x = -10.0
	if is_top(direction):
		sprite.position.y = -6.0
	elif is_bottom(direction):
		sprite.position.y = 6.0

	match direction:
		Direction.TopRight: sprite.rotation = 0.0
		Direction.TopLeft: sprite.rotation = -PI / 2.0
		Direction.BottomRight: sprite.rotation = PI / 2.0
		Direction.BottomLeft: sprite.rotation = PI

@export var spring_force := 16.0:
	set(value):
		spring_force = value
		queue_redraw()
@export var sprite: AnimatedSprite2D

func check_horizontal_conditions(player: Character) -> bool:
	return is_right(direction) and player.global_position.x > global_position.x - 4.0 or \
		is_left(direction) and player.global_position.x < global_position.x + 4.0

func spring(player: Character) -> void:
	if is_top(direction):
		player.clear_standing_objects()
		player.has_jumped = false
		player.set_state("SpringBounce")
		player.spring_bounce_timer = 48

	var vector := direction_vector()
	player.global_position -= vector * 8.0
	player.velocity = vector * spring_force
	var animation := sprite.animation.replace("relaxed", "spring")
	sprite.play(animation)


func _on_animated_sprite_2d_animation_finished() -> void:
	if sprite.animation.begins_with("spring"):
		var animation := sprite.animation.replace("spring", "relaxed")
		sprite.play(animation)
