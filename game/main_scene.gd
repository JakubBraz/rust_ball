extends Node3D

var socket
var start_touch = Vector2()
var prev_touch = Vector2()

const SCALING = 0.5

# Called when the node enters the scene tree for the first time.
func _ready():
	socket = PacketPeerUDP.new()
	socket.set_dest_address("127.0.0.1", 8019)
	socket.put_packet("hello".to_ascii_buffer())
	print("position: ", $CSGSphere3D.position)
	print("position: ", $CSGSphere3D2.position)

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	#print("process " + str(delta))
	#socket.put_packet("test".to_ascii_buffer())
	if Input.is_action_pressed("ui_left"):
		$CSGSphere3D.position += Vector3(-0.15, 0.0, 0.0)
	elif Input.is_action_pressed("ui_right"):
		$CSGSphere3D.position += Vector3(0.15, 0.0, 0.0)
	elif Input.is_action_pressed("ui_up"):
		$CSGSphere3D.position += Vector3(0.0, 0.0, -0.15)
	elif Input.is_action_pressed("ui_down"):
		$CSGSphere3D.position += Vector3(0.0, 0.0, 0.15)
	
	if start_touch != Vector2():
		var pos = get_viewport().get_mouse_position()
		var d = (prev_touch - pos).length()
		#print("distance", d)
		if d > 5:
			print("touch ", pos)
			prev_touch = pos
			var bytes  = PackedByteArray([0, 0, 0, 0, 0, 0, 0, 0])
			bytes.encode_u16(0, start_touch[0])
			bytes.encode_u16(2, start_touch[1])
			bytes.encode_u16(4, pos[0])
			bytes.encode_u16(6, pos[1])
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
		print([player_x, player_y, player_r, ball_x, ball_y, ball_r])
		$CSGSphere3D.position = Vector3(player_x * SCALING, $CSGSphere3D.position[1], player_y * SCALING)
		$CSGSphere3D2.position = Vector3(ball_x * SCALING, $CSGSphere3D2.position[1], ball_y * SCALING)

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
