extends Area2D

func _on_area_entered(area: Area2D) -> void:
	var player_hitbox = area as PlayerHitbox
	if player_hitbox:
		player_hitbox.on_hurt(self)

