extends Tool

@export var scene:PackedScene

func _ready() -> void:
	setup()
	tool_used.connect(use_tool)

func use_tool() -> void:
	var position := get_global_mouse_position()
	var node :Node2D= scene.instantiate()
	node.global_position = position
	get_tree().current_scene.add_child(node)
