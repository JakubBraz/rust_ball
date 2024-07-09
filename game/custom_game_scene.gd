extends Control

var digits = random_digits()
var global_values

func _ready():
	global_values = get_node("/root/GlobalValues")
	set_labels()
	$RandomDigitsButton.pressed.connect(new_digits)
	$JoinGameButton.pressed.connect(func(): global_values.change_scene(room_id()))
	$Digit1.pressed.connect(func(): new_digit(0))
	$Digit2.pressed.connect(func(): new_digit(1))
	$Digit3.pressed.connect(func(): new_digit(2))
	$Digit4.pressed.connect(func(): new_digit(3))

func room_id():
	return digits[0] * 1000 + digits[1] * 100 + digits[2] * 10 + digits[3]

func random_digits():
	return [randi_range(0, 9), randi_range(0, 9), randi_range(0, 9), randi_range(0, 9)]

func set_labels():
	$Digit1.text = str(digits[0])
	$Digit2.text = str(digits[1])
	$Digit3.text = str(digits[2])
	$Digit4.text = str(digits[3])

func new_digits():
	digits = random_digits()
	set_labels()
	
func new_digit(i):
	digits[i] += 1
	if digits[i] >= 10:
		digits[i] = 0
	set_labels()
