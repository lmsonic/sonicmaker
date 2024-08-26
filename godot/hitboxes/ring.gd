class_name Ring extends Area2D

@export var amount := 1

@export var sprite: AnimatedSprite2D
var collected := false
func _on_area_entered(area: Area2D) -> void:
	if collected:
		return
	var player_hitbox := area as PlayerHitbox
	if player_hitbox and player_hitbox.can_gather_rings():
		player_hitbox.increment_rings(amount)
		sprite.play("collected")
		collected = true
		z_index = 1
		await sprite.animation_finished
		queue_free()
