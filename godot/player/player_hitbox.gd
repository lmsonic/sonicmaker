class_name PlayerHitbox extends Area2D

@export var player:Character

@export var attacking := false

func increment_rings(amount:int):
	player.rings += amount
	EventBus.rings_set.emit(player.rings)

func on_hurt(hazard:Node2D):
	player.on_hurt(hazard)
	EventBus.rings_set.emit(player.rings)

func on_attacking_badnik(badnik:Node2D):
	if attacking:
		player.on_attacking(badnik,false)
	else:
		on_hurt(badnik)

func on_attacking_boss(boss:Node2D):
	if attacking:
		player.on_attacking(boss,true)
	else:
		on_hurt(boss)
