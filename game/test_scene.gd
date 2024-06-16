extends Node3D

var press1 = true
var press2 = true
var press3 = true

#func _ready():
	#$container/RigidBody3D4.add_constant_central_force(Vector3(-1.0, 0.0, 0.0))

# TO DZIALA!
#func _physics_process(delta):
	#$container.position += Vector3(0.01, 0.0, 0.0)

func _process(delta):
	#$container.position += Vector3(0.001, 0.0, 0.0)
	if Input.is_action_pressed("ui_up") and press1:
		print("JUMP")
		#$RigidBody3D.add_constant_torque(Vector3(0.0, 5.0, 0.0))
		$container/RigidBody3D.apply_torque_impulse(Vector3(5.0, 1.0, 0.0))
		press1 = false
	elif Input.is_action_just_released("ui_up"):
		press1 = true
	elif Input.is_action_pressed("ui_right") and press2:
		print("JUMP")
		$RigidBody3D2.apply_central_impulse(Vector3(3.0, 0.0, 0.0))
		press2 = false
	elif Input.is_action_just_released("ui_right"):
		press2 = true
	elif Input.is_action_pressed("ui_left") and press2:
		print("JUMP")
		$RigidBody3D2.apply_central_impulse(Vector3(-3.0, 0.0, 0.0))
		press2 = false
	elif Input.is_action_just_released("ui_left"):
		press2 = true
	elif Input.is_action_pressed("ui_select") and press3:
		print("JUMP")
		$container/RigidBody3D4.apply_central_impulse(Vector3(0.5, 0.0, 0.0))
		press3 = false
	elif Input.is_action_just_released("ui_select"):
		press3 = true
