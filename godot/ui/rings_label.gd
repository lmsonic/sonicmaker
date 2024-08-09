extends Label


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	EventBus.increment_rings.connect(on_increment_rings)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func on_increment_rings(_value: int) -> void:
	text = "Rings: %s" % [EventBus.rings]
