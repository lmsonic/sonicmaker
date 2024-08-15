extends SolidObject

func _physics_process(delta: float) -> void:
	physics_process(delta)

	var player :Character = get_tree().get_first_node_in_group("player") as Character
	if !player:return
	match collision :
		"Up":
			player.on_hurt(self)


