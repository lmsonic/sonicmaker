extends Tool
@onready var cursor: Sprite2D = %Cursor

@export var scene: PackedScene

@export var objects: Node2D

func _ready() -> void:
	setup()
	tool_used.connect(use_tool)


func use_tool() -> void:
	var position := get_global_mouse_position().snapped(tile_size)
	var node: Node2D = scene.instantiate()
	node.global_position = position
	var spring:= node as Spring
	if spring:
		spring.direction = tool_direction
		node.position -= spring.sprite.position
	var spike:= node as Spike
	if spike:
		spike.direction = tool_direction




	objects.add_child(node)
