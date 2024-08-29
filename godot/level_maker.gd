extends Node2D

enum ToolSelected {
	Player,
	Ground,
	Ring,
	Spike,
	YellowSpring,
	RedSpring
}
@onready var tilemap: TileMapLayer = $TileMapLayer
@onready var cursor: Marker2D = $Cursor
const PLAYER = preload("res://player/character.tscn")
const PLAYER_CURSOR = preload("res://ui/player_cursor.tscn")
const RING = preload("res://hitboxes/ring.tscn")
const SPRING = preload("res://solid_objects/spring.tscn")
const RED_SPRING = preload("res://solid_objects/red_spring.tscn")
const SPIKE = preload("res://solid_objects/spike.tscn")
const GROUND = preload("res://ui/ground_cursor.tscn")
var tool := ToolSelected.Ring:
	set(value):
		if tool == value:
			return
		tool = value
		update_cursor_scene()


func tool_snap() -> float:
	match tool:
		ToolSelected.Ring | ToolSelected.Ground: return 16
		ToolSelected.Player | ToolSelected.YellowSpring | ToolSelected.RedSpring | ToolSelected.Spike: return 32
	return 16

func can_rotate() -> bool:
	match tool:
		ToolSelected.YellowSpring | ToolSelected.RedSpring | ToolSelected.Spike: return true
	return false

func update_cursor_scene() -> void:
	var new_scene := scene_from_tool()
	if cursor_scene:
		cursor_scene.queue_free()
		cursor_scene = null
	if new_scene:
		new_scene.modulate.a = 0.5
		cursor_scene = new_scene
		cursor_scene.process_mode = Node.PROCESS_MODE_DISABLED
		cursor.add_child(cursor_scene)

func scene_from_tool() -> Node2D:
	match tool:
		ToolSelected.Player: return PLAYER_CURSOR.instantiate()
		ToolSelected.Ring: return RING.instantiate()
		ToolSelected.YellowSpring: return SPRING.instantiate()
		ToolSelected.RedSpring: return RED_SPRING.instantiate()
		ToolSelected.Spike: return SPIKE.instantiate()
		ToolSelected.Ground: return GROUND.instantiate()
	return null
@onready var cursor_scene: Node2D = null

func action_from_tool() -> void:
	var mouse_position := get_global_mouse_position()
	match tool:
		ToolSelected.Player:
			var player:Character= get_tree().get_first_node_in_group("player")
			if player:
				player.global_position = mouse_position
			else:
				player = PLAYER.instantiate()
				player.global_position = mouse_position
				add_child(player)
		ToolSelected.Ring:
			var node := scene_from_tool()
			node.global_position = mouse_position
			add_child(node)
		ToolSelected.YellowSpring:
			var node := scene_from_tool()
			node.global_position = mouse_position
			add_child(node)
		ToolSelected.RedSpring:
			var node := scene_from_tool()
			node.global_position = mouse_position
			add_child(node)
		ToolSelected.Spike:
			var node := scene_from_tool()
			node.global_position = mouse_position
			add_child(node)
		ToolSelected.Ground:
			var local_mouse_position := get_local_mouse_position()
			var cell_position := tilemap.local_to_map(mouse_position)
			tilemap.set_cells_terrain_connect([cell_position], 0, 0)

func _ready() -> void:
	update_cursor_scene()
	Input.use_accumulated_input = false

func _process(delta: float) -> void:
	var mouse_position := get_global_mouse_position()
	cursor.global_position = mouse_position.snappedf(tool_snap())


func _unhandled_input(event: InputEvent) -> void:
	if Input.is_action_just_pressed("click"):
		action_from_tool()


func _on_player_button_pressed() -> void:
	tool = ToolSelected.Player

func _on_ground_button_pressed() -> void:
	tool = ToolSelected.Ground

func _on_ring_button_pressed() -> void:
	tool = ToolSelected.Ring

func _on_spike_button_pressed() -> void:
	tool = ToolSelected.Spike

func _on_yellow_spring_button_pressed() -> void:
	tool = ToolSelected.YellowSpring

func _on_red_spring_button_pressed() -> void:
	tool = ToolSelected.RedSpring
