import cmd
import sys, tty, termios
import socket

# This function allows reading stdin in raw mode (meaning, without expecting the '\n' to transmit the data)
# only one character per character
def read_char():
	fd = sys.stdin.fileno()
	old_tty_setting = termios.tcgetattr(fd)
	new_tty_setting = termios.tcgetattr(fd)
	try:
		tty.setraw(fd, termios.TCSADRAIN)
		return sys.stdin.read(1)
	finally:
		termios.tcsetattr(fd, termios.TCSADRAIN, old_tty_setting)

# This function ensures the argument is not empty, neither multiple instructions.
def check_arg(arg):
	print(arg)
	if (len(arg) == 0 or (' ' in arg)):
		print(FORMAT)
		return False
	else:
		return True

# This function convert the string passed as argument to a bytes-like format,
# then send it to the server.
def	send_data(str, sock):
	data = str
	data_bytes = data.encode('utf-8')
	sock.send(data_bytes)

class InputInterpretor(cmd.Cmd):
	prompt = 'Taskmaster > '

	def	__init__(self, sock):
		super().__init__()
		self.sock = sock

	def	default(self, arg):
		print(FORMAT)
	
	def do_start(self, arg):
		'Start the program specified in the argument.\n'
		if check_arg(arg):
			send_data("start", sock)
			print("Start command :) ")
	
	def do_stop(self, arg):
		'Stop the program specified in the argument.\n'
		if check_arg(arg):
			send_data("stop", sock)
			print("Stop command")

	def do_restart(self, arg=None):
		'Start the program specified in the argument.\n'
		if check_arg(arg):
			send_data("restart", sock)
			print("Restart command")
	
	def do_quit(self, arg):
		'Disconnect the client and quit the program.\n'
		print('Client deconnexion')
		sock.close()
		return True

	def do_update(self, arg):
		'Update the config file.\n'
		send_data("update", sock)
		print('updating the config file')

	def	emptyline(self):
		pass

	def cmd_loop(self):
		while (42):
			char = read_char()
			self.cmdloop(char)

HOST = '127.0.0.1'
PORT = 4241
FORMAT = "Correct format is : [command] [program]\nType 'help' for all commands.\n"

if __name__ == "__main__":
	sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
	sock.connect((HOST, PORT))
	print('Connected')
	InputInterpretor(sock).cmdloop()
