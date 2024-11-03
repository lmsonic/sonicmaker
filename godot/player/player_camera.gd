# From https://info.sonicretro.org/SPG:Camera
extends Camera2D

@export var player: Character

const air_vertical_margin := 0.14285714285

static func move_toward_positioni(origin: Vector2, target: Vector2, speed: float) -> Vector2:
	target = floor(target)
	origin = floor(origin)
	return Vector2(move_toward(target.x, origin.x, speed), move_toward(target.y, origin.y, speed))

func _physics_process(_delta: float) -> void:
	if player.is_grounded:
		drag_top_margin = 0.0
		drag_bottom_margin = 0.0
		position_smoothing_speed = 24.0 if absf(player.ground_speed) >= 8.0 else 6.0

	else:
		drag_top_margin = air_vertical_margin
		drag_bottom_margin = air_vertical_margin
		position_smoothing_speed = 24
	position = move_toward_positioni(position, offset, position_smoothing_speed)
