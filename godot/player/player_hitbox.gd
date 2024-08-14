class_name PlayerHitbox extends Area2D

@export var player:Character
var regather_rings_timer := 0
func _ready() -> void:
	await get_tree().process_frame
	EventBus.rings_set.emit(player.rings)

func can_gather_rings() -> bool:
	return regather_rings_timer <= 0

func _physics_process(delta: float) -> void:
	if regather_rings_timer > 0:
		regather_rings_timer -= 1

func increment_rings(amount:int) -> void:
	player.rings += amount
	EventBus.rings_set.emit(player.rings)

func on_hurt(hazard:Node2D) -> void:
	await get_tree().process_frame
	regather_rings_timer = 64
	player.on_hurt(hazard)
	EventBus.rings_set.emit(player.rings)

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
