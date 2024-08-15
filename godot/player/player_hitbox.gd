class_name PlayerHitbox extends Area2D

@export var player:Character

func _ready() -> void:
	await get_tree().process_frame
	_on_rings_changed(player.rings)

func _on_rings_changed(value: int) -> void:
	EventBus.rings_set.emit(value)

func can_gather_rings() -> bool:
	return player.can_gather_rings()

func increment_rings(amount:int) -> void:
	player.rings += amount

func on_hurt(hazard:Node2D) -> void:
	await get_tree().process_frame
	player.on_hurt(hazard)

func on_attacking_badnik(badnik:Node2D) -> void:
	if player.attacking:
		player.on_attacking(badnik,false)
	else:
		on_hurt(badnik)

func on_attacking_boss(boss:Node2D) -> void:
	if player.attacking:
		player.on_attacking(boss,true)
	else:
		on_hurt(boss)


