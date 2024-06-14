extends Node2D

var rotation_speed = PI / 4

# Called when the node enters the scene tree for the first time.
func _ready():
	var player_id = get_node("/root/GlobalValues").player_id
	print(player_id)

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	$Sprite2D.rotate(rotation_speed * delta)

func _input(event):
	if (event is InputEventMouseButton):
		get_node("/root/GlobalValues").player_id += 1
		get_tree().change_scene_to_file("res://main_scene.tscn")
