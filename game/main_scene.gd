extends Node3D

var socket
var start_touch = Vector2()
var prev_touch = Vector2()


# Called when the node enters the scene tree for the first time.
func _ready():
	socket = PacketPeerUDP.new()
	socket.set_dest_address("127.0.0.1", 8019)
	socket.put_packet("hello".to_ascii_buffer())
	print("position: ", $player.position)
	print("position: ", $ball.position)
	

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	#print("process " + str(delta))
	#socket.put_packet("test".to_ascii_buffer())
	
	if Input.is_action_pressed("ui_left"):
		$player.position += Vector3(-0.15, 0.0, 0.0)
	elif Input.is_action_pressed("ui_right"):
		$player.position += Vector3(0.15, 0.0, 0.0)
	elif Input.is_action_pressed("ui_up"):
		$player.position += Vector3(0.0, 0.0, -0.15)
	elif Input.is_action_pressed("ui_down"):
		$player.position += Vector3(0.0, 0.0, 0.15)
	
	if start_touch != Vector2():
		var pos = get_viewport().get_mouse_position()
		var d = (prev_touch - pos).length()
		#print("distance", d)
		if d > 1:
			print("touch ", pos)
			prev_touch = pos
			var touch_vec = Vector2(pos[0] - start_touch[0], pos[1] - start_touch[1])
			touch_vec = touch_vec.rotated(-get_viewport().get_camera_3d().rotation[1])
			var bytes  = PackedByteArray([0, 0, 0, 0, 0, 0, 0, 0])
			#bytes.encode_u16(0, start_touch[0])
			bytes.encode_s16(0, 0)
			#bytes.encode_u16(2, start_touch[1])
			bytes.encode_s16(2, 0)
			bytes.encode_s16(4, touch_vec[0])
			bytes.encode_s16(6, touch_vec[1])
			socket.put_packet(bytes)
			#socket.put_packet("aaa".to_ascii_buffer())
	#if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT):
		#print("MOUSE PRESSED")

	var p = socket.get_packet()
	if len(p) > 0:
		print("get_packet: ", p, " ", p.get_string_from_ascii())
		var player_x = p.decode_float(0);
		var player_y = p.decode_float(4);
		var player_r = p.decode_float(8);
		var ball_x = p.decode_float(12);
		var ball_y = p.decode_float(16);
		var ball_r = p.decode_float(20);
		var vector_x = p.decode_float(24);
		var vector_y = p.decode_float(28);
		print([player_x, player_y, player_r, ball_x, ball_y, ball_r])
		$player.position = Vector3(player_x, $player.position[1], player_y)
		$ball.position = Vector3(ball_x, $ball.position[1], ball_y)
		var touch_vec = Vector2(vector_x, vector_y)
		$vector_container.scale = Vector3(touch_vec.length(), 1, 1)
		$vector_container.rotation = Vector3(0, -touch_vec.angle(), 0)
		$vector_container.position = $player.position

func _input(event):
	if (event is InputEventMouseButton):
		if event.pressed:
			print("pressed:")
			print(event.position)
			start_touch = event.position
		else:
			print("released")
			print(event.position)
			start_touch = Vector2()
			socket.put_packet(PackedByteArray([0, 0, 0, 0, 0, 0, 0, 0]))
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
