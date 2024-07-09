extends Node3D

var global_values

var tcp_socket: StreamPeerTCP
var connected_to_host = false

func _ready():
	print("tcp handler ready")
	global_values = get_node("/root/GlobalValues")
	print("Connecting TCP %s %s" % [global_values.IP_ADDR, global_values.TCP_PORT])
	tcp_socket = StreamPeerTCP.new()
	tcp_socket.set_no_delay(true)
	tcp_socket.connect_to_host(global_values.IP_ADDR, global_values.TCP_PORT)

func _process(delta):
	if connected_to_host == false:
		tcp_socket.poll()
		if tcp_socket.get_status() == 2:
			print("Connected to host")
			connected_to_host = true
			#print("TCP STATUS: %s" % tcp_status(tcp_socket.get_status()))
			#todo pick room id
			tcp_socket.put_data([123,123])
	
	#var result = tcp_socket.put_data(PackedByteArray([113, 113, 202, 223]))
	##var result = tcp_socket.put_data("ab".to_ascii_buffer())
	#if result != 1:
		#print("Send result: ", result)
	
	if connected_to_host:
		if tcp_socket.get_available_bytes() > 0:
			print('reading socket...')
			global_values.player_id = tcp_socket.get_64()
			print('reading done: ', global_values.player_id)
			global_values.send_input(0, Vector2())

func tcp_status(status):
	if status == 0:
		return "STATUS_NONE"
	if status == 1:
		return "STATUS_CONNECTING"
	if status == 2:
		return "STATUS_CONNECTED"
	if status == 3:
		return "STATUS_ERROR"
	return "UNKOWN, status " + str(status)
