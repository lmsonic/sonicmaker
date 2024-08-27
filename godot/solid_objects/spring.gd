@tool
extends SolidObject

enum Direction {
	Right,
	Up,
	Left,
	Down
}

@export var direction := Direction.Up:
	set(value):
		direction = value
		if direction == Direction.Right || direction == Direction.Left:
			width_radius = 8
			height_radius = 14
		else:
			width_radius = 16
			height_radius = 8
@export var spring_force := 16.0

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

func _physics_process(delta: float) -> void:
	physics_process(delta)

	var player: Character = get_tree().get_first_node_in_group("player") as Character
	if !player: return
	if collision_matches_direction(collision):
		if direction == Direction.Up || direction == Direction.Down:
			player.clear_standing_objects()
			player.velocity.y = -spring_force if direction == Direction.Up else spring_force
			player.global_position.y += 8 if direction == Direction.Up else -8
			player.set_state("SpringBounce")
			player.spring_bounce_timer = 48
		else:

			if player.is_grounded:
				player.ground_speed = spring_force if direction == Direction.Right else -spring_force
			else:
				player.global_position.x += -8 if direction == Direction.Right else 8
			player.global_position.x += -8 if direction == Direction.Right else 8
			player.set_flip_h(direction == Direction.Left)
			player.control_lock_timer = 16


func check_box_around_player(player: Character) -> bool:
	var box := Rect2(global_position, Vector2(40.0, 24.0))
	return box.has_point(player.global_position)

func is_player_moving_towards_spring(player: Character) -> bool:
	if player.velocity.x == 0.0:
		return false
	var delta := global_position - player.global_position
	return player.velocity.x != 0.0 and signf(player.velocity.x) == signf(delta.x)
