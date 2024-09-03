extends Area2D

signal attacked

func _on_area_entered(area: Area2D) -> void:
	var player_hitbox := area as PlayerHitbox
	if player_hitbox:
		var player := player_hitbox.player
		if player.attacking:
			player_hitbox.on_attacking_badnik(self)
			attacked.emit()
		else:
			player_hitbox.on_hurt(self)
