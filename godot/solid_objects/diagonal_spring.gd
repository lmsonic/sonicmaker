@tool extends SlopedSolidObject

enum Direction {
	TopRight,
	TopLeft,
	BottomRight,
	BottomLeft
}

func is_bottom(dir: Direction) -> bool:
	return dir == Direction.BottomRight || dir == Direction.BottomLeft

func is_top(dir: Direction) -> bool:
	return dir == Direction.TopRight || dir == Direction.TopLeft

func is_right(dir: Direction) -> bool:
	return dir == Direction.BottomRight || dir == Direction.TopRight

func is_left(dir: Direction) -> bool:
	return dir == Direction.TopLeft || dir == Direction.BottomLeft


@export var direction := Direction.TopRight:
	set(value):
		if is_bottom(direction) and is_top(value) || is_bottom(value) and is_top(direction):
			flip_y()
		if is_left(direction) and is_right(value) || is_left(value) and is_right(direction):
			flip_x()
		direction = value

func _physics_process(delta: float) -> void:
	physics_process(delta)
