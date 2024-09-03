extends SolidObject

@export var moving := false

var timer := 0
var extended := false
@onready var original_y := global_position.y

func _physics_process(delta: float) -> void:
	physics_process(delta)
	if moving:
		timer += 1
		if timer >= 64:
			move()

func move() -> void:
	if extended:
		# Retracting
		position.y += 8
		if position.y >= original_y:
			timer = 0
			extended = false
	else:
		# Extending
		position.y -= 8
		if position.y <= original_y - 32:
			timer = 0
			extended = true


func _on_collided(collision: String, player: Character) -> void:
	match collision:
		"Up":
			player.on_hurt(self)
