extends Area2D
@onready var sprite: AnimatedSprite2D = $AnimatedSprite2D

func _on_area_entered(area: Area2D) -> void:
	var player_hitbox := area as PlayerHitbox
	if player_hitbox:
		var player := player_hitbox.player
		var delta := player.global_position - global_position
		player.velocity = delta.normalized() * 7
		sprite.play("bump")


func _on_animated_sprite_2d_animation_finished() -> void:
	sprite.play("default")
	sprite.stop()
