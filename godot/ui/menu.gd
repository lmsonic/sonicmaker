extends Control


func _on_demo_button_pressed() -> void:
	get_tree().change_scene_to_file("res://main.tscn")


func _on_level_maker_button_pressed() -> void:
	get_tree().change_scene_to_file("res://ui/level_maker.tscn")


func _on_quit_button_pressed() -> void:
	get_tree().quit()
