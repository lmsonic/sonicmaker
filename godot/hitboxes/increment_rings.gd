extends Area2D

@export var amount := 1

func _on_area_entered(area: Area2D) -> void:
	var player_hitbox := area as PlayerHitbox
	if player_hitbox:
		player_hitbox.increment_rings(amount)
		queue_free()
