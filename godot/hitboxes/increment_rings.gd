extends Area2D

@export var amount := 1
@onready var sprite: AnimatedSprite2D = $AnimatedSprite2D
var collected:=false
func _on_area_entered(area: Area2D) -> void:
	if collected:
		return
	var player_hitbox := area as PlayerHitbox
	if player_hitbox:
		player_hitbox.increment_rings(amount)
		sprite.play("collected")
		collected = true
		await sprite.animation_finished
		queue_free()

