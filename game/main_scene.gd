extends Node3D

# todo make this value screen widht/height dependent
const MAX_TOUCH_LEN_SCREEN = 70.0
const TOUCH_SCALLING = 0.05

var joypad_vector = Vector2()

var start_touch = Vector2()
var prev_touch = Vector2()
var message_id = 0
var global_values

# Called when the node enters the scene tree for the first time.
func _ready():
	print("position: ", $player.position)
	print("position: ", $ball.position)
	global_values = get_node("/root/GlobalValues")
	print(global_values.player_id)

func _process(delta):
	#if Input.is_action_pressed("ui_left"):
		#$player.position += Vector3(-0.15, 0.0, 0.0)
	#elif Input.is_action_pressed("ui_right"):
		#$player.position += Vector3(0.15, 0.0, 0.0)
	#elif Input.is_action_pressed("ui_up"):
		#$player.position += Vector3(0.0, 0.0, -0.15)
	#elif Input.is_action_pressed("ui_down"):
		#$player.position += Vector3(0.0, 0.0, 0.15)
	
	if start_touch != Vector2():
		var pos = get_viewport().get_mouse_position()
		var touch_vec = pos - start_touch
		var len = min(MAX_TOUCH_LEN_SCREEN, touch_vec.length())
		var normalized_touch = touch_vec.normalized() * (len / MAX_TOUCH_LEN_SCREEN)
		$vector_container.scale = Vector3(len * TOUCH_SCALLING, 1, 1)
		#$vector_container.scale = Vector3(MAX_TOUCH_LEN_GAME, 1, 1)
		$vector_container.rotation = Vector3(0, -touch_vec.angle(), 0)
		$vector_container.position = $player.position
		
		var d = (prev_touch - pos).length()
		#print("distance", d)
		if d > 1:
			print("touch ", pos)
			prev_touch = pos
			message_id += 1
			global_values.send_input(message_id, normalized_touch)
		
	#if joypad_vector.length() > 0.2:
		#message_id += 1
		#global_values.send_input(message_id, joypad_vector)

	# todo move reading packet to global_values
	var p = global_values.socket.get_packet()
	if len(p) > 0:
		print(global_values.game_time, ", get_packet: ", p, " ", p.get_string_from_ascii())
		var player_x = p.decode_float(16);
		var player_y = p.decode_float(20);
		var player2_x = p.decode_float(32);
		var player2_y = p.decode_float(36);
		var ball_x = p.decode_float(8);
		var ball_y = p.decode_float(12);
		#print([player_x, player_y, player_r, ball_x, ball_y, ball_r])
		$player.position = Vector3(player_x, $player.position[1], player_y)
		$player2.position = Vector3(player2_x, $player2.position[1], player2_y)
		$ball.position = Vector3(ball_x, $ball.position[1], ball_y)
		

func _input(event):
	if (event is InputEventMouseButton):
		if event.pressed:
			print("pressed:")
			print(event.position)
			start_touch = event.position
			$vector_container.visible = true
			$touch_icon.visible = true
			$touch_icon.position = start_touch
		else:
			print("released")
			print(event.position)
			start_touch = Vector2()
			$vector_container.visible = false
			$touch_icon.visible = false
			message_id += 1
			global_values.send_input(message_id, start_touch)
	elif (event is InputEventKey) and not event.is_echo():
		if Input.is_key_pressed(KEY_1):
			$Camera3D1.current = true
		elif Input.is_key_pressed(KEY_2):
			$Camera3D2.current = true
		elif Input.is_key_pressed(KEY_3):
			$Camera3D3.current = true
		elif Input.is_key_pressed(KEY_4):
			$Camera3D4.current = true
		elif Input.is_key_pressed(KEY_5):
			$Camera3D5.current = true
		elif Input.is_key_pressed(KEY_6):
			$Camera3D6.current = true
		elif Input.is_key_pressed(KEY_7):
			$Camera3D7.current = true
	elif (event is InputEventJoypadMotion):
		#print(event)
		#print(Input.get_vector("move_left", "move_right", "move_forward", "move_back"))
		if event.axis == 0:
			joypad_vector[0] = event.axis_value
		elif event.axis == 1:
			joypad_vector[1] = event.axis_value
