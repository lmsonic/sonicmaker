@tool extends SlopedSolidObject

enum Direction {
	TopRight,
	TopLeft,
	BottomRight,
	BottomLeft
}

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
		Direction.TopRight: return Vector2(0.707107, -0.707107)
		Direction.TopLeft: return Vector2(-0.707107, -0.707107)
		Direction.BottomRight: return Vector2(0.707107, 0.707107)
		Direction.BottomLeft: return Vector2(-0.707107, 0.707107)
	return Vector2.ZERO

func _draw() -> void:
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

@export var spring_force := 16.0


func _physics_process(delta: float) -> void:
	physics_process(delta)
	if Engine.is_editor_hint():
		return
	var player: Character = get_tree().get_first_node_in_group("player") as Character
	if !player: return

	match collision:
		"Up":
			if is_top(direction) and check_horizontal_conditions(player):
				spring(player)
		"Down":
			if is_bottom(direction) and check_horizontal_conditions(player):
				spring(player)

func check_horizontal_conditions(player: Character) -> bool:
	return is_right(direction) and player.global_position.x > global_position.x - 4.0 or \
		is_left(direction) and player.global_position.x < global_position.x + 4.0

func spring(player: Character) -> void:
	if is_top(direction):
		player.clear_standing_objects()
		player.has_jumped = false
	var vector := direction_vector()

	print("springed")
	player.global_position -= vector * 8.0
	player.velocity = vector * spring_force
