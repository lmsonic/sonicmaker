extends Tool
@export var tilemap_layer:TileMapLayer
@export var terrain_set:=0
@export var terrain:=0
func _ready() -> void:
	setup()
	tool_used.connect(use_tool)

func use_tool() -> void:
	var mouse_position := get_global_mouse_position()
	var map_position:= tilemap_layer.local_to_map(mouse_position)
	tilemap_layer.set_cells_terrain_connect([map_position],terrain_set,terrain,false)
