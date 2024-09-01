extends Node2D

var time := 0.0

var radius := 50.0
@onready var original_pos := global_position

func _process(delta: float) -> void:
	global_position.x = original_pos.x + cos(time) * radius
	global_position.y = original_pos.y + sin(time) * radius
	time += delta
