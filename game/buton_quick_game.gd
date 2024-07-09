extends Button

var global_values

func _ready():
	global_values = get_node("/root/GlobalValues")
	pressed.connect(func(): global_values.change_scene(global_values.QUICK_GAME_ROOM_ID))
