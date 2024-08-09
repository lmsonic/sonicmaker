extends Area2D

@export var amount := 1

func _on_area_entered(area: Area2D) -> void:
	if area is PlayerHitbox:
		EventBus.increment_rings.emit(amount)
		queue_free()
