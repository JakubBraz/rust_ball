extends Node

var socket
var ping_freq = 10.0
var packet_out = PackedByteArray([
	13, 178, # constant value, always the same
	0, 0, 0, 0, 0, 0, 0, 0, # player id, given by server in tcp message
	0, 0, 0, 0, # message id
	0, 0, 0, 0, # normalized touch vec x
	0, 0, 0, 0, # normalized touch vec y
	0, 0, 0, 0, 0, 0, 0, 0, 0, 0, #unused
	])

var player_id = 4_023_432_123
var game_time = 0.0
var last_ping_time = -30.0
var packet_id = 0

var room_id = 12123

const IP_ADDR = "127.0.0.1"
const UDP_PORT = 8019
const TCP_PORT = 8018

# Called when the node enters the scene tree for the first time.
func _ready():
	print("Game start!")
	socket = PacketPeerUDP.new()
	#socket.set_dest_address("127.0.0.1", 8019)
	socket.set_dest_address("20.215.53.164", 8019)
	#socket.set_dest_address("172.27.181.206", 8019)
	#socket.connect_to_host("127.0.0.1", 8019)

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	game_time += delta
	#if game_time - last_ping_time > ping_freq:
		#packet_id += 1
		#print("sending ping ...")
		#var bytes = packet_out
		#bytes.encode_u16(2, 1)
		#bytes.encode_u32(4, player_id)
		#bytes.encode_u32(8, packet_id)
		#bytes.encode_u16(20, room_id)
		#print(bytes)
		#socket.put_packet(bytes)
		#last_ping_time = game_time

func send_input(message_id, v):
	var bytes  = packet_out
	bytes.encode_u64(2, player_id)
	bytes.encode_u32(10, message_id)
	# todo dont send those zeros, the same as previous todo
	bytes.encode_float(14, v[0])
	bytes.encode_float(18, v[1])
	socket.put_packet(bytes)
