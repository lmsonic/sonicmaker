extends Node
signal increment_rings(value:int)

var rings := 0

func _ready() -> void:
	increment_rings.connect(func (value): rings+=value)
