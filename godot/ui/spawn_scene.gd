extends Tool

@export var scene:PackedScene

@export var objects:Node2D
func _ready() -> void:
	setup()
	tool_used.connect(use_tool)

func use_tool() -> void:
	var position := get_global_mouse_position().snapped(tile_size)
	var node :Node2D= scene.instantiate()
	node.global_position = position
	objects.add_child(node)
