extends Label


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	EventBus.rings_set.connect(on_rings_set)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func on_rings_set(value: int) -> void:
	text = "Rings: %s" % [value]
