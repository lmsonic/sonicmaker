extends Camera2D

@export var player:Character

const air_vertical_margin := 0.14285714285

func _physics_process(delta: float) -> void:
	if player.is_grounded:
		drag_top_margin = 0.0
		drag_bottom_margin = 0.0
		#if absf(player.ground_speed) >= 8.0:
			#position_smoothing_speed = 24.0
		#else:
			#position_smoothing_speed = 6.0
	else:
		drag_top_margin = air_vertical_margin
		drag_bottom_margin = air_vertical_margin
		#position_smoothing_speed = 24.0
