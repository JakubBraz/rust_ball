extends Button


func _ready():
	pressed.connect(change_scene)

#func _process(delta):
	#pass

func change_scene():
	print("Changing scene")
	get_tree().change_scene_to_file("res://custom_game_scene.tscn")
