extends Tool

@export var scene:PackedScene
@export var objects:Node2D

func _ready() -> void:
	setup()
	tool_used.connect(use_tool)

func use_tool() -> void:
	var player:Character= get_tree().get_first_node_in_group("player")


	var position := get_global_mouse_position().snapped(tile_size)
	if player:
		player.global_position = position
		player.velocity = Vector2.ZERO
	else:
		var node :Character= scene.instantiate()
		node.global_position = position
		objects.add_child(node)
		var camera:=node.get_node("Camera2D")
		if camera:
			camera.set_physics_process(false)
