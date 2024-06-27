import cmd
import sys, tty, termios
import socket
import signal
import sys
import os


HOST = 'localhost'
PORT = 4241

# This function allows reading stdin in raw mode (meaning, without expecting the '\n' to transmit the data)
# only one character per character
def read_char():
	fd = sys.stdin.fileno()
	old_tty_setting = termios.tcgetattr(fd)
	try:
		tty.setraw(fd, termios.TCSADRAIN)
		return sys.stdin.read(1)
	finally:
		termios.tcsetattr(fd, termios.TCSADRAIN, old_tty_setting)

def	send_data(str):
	try:
		sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
		sock.connect((HOST, PORT))
		sock.send(str.encode('utf-8'))
		while (True):
			receive = sock.recv(128).decode('utf-8')
			if not receive:
				break
			print(receive, end='')
		print("")
		sock.close()
	except Exception as e:
		print("Error when sending data:", e)

class InputInterpretor(cmd.Cmd):
	prompt = 'Taskmaster > '

	def	__init__(self):
		super().__init__()

	def	default(self, line):
		print(f"{line} is invalid...")
		print("Correct format is: [command] [program]")
		print("Type 'help' for all commands.")
	
	def do_status(self, arg):
		"""Print the status of all the programs"""
		send_data(f"status {arg}")

	def do_start(self, arg):
		"""Start the program specified in argument"""
		send_data(f"start {arg}")
	
	def do_stop(self, arg):
		"""Stop the program specified in argument"""
		send_data(f"stop {arg}")

	def do_restart(self, arg=None):
		"""Restart the program specified in argument"""
		send_data(f"restart {arg}")
	
	def do_quit(self, arg):
		"""Disconnect the client and quit program"""
		sys.exit(0)

	def do_reload(self, arg):
		"""Reload the config file"""
		os.system("pkill -hup taskmaster")
		print("taskmaster config file is reloaded")

	def do_kill(self, arg):
		"""Reload the config file"""
		os.system("pkill -int taskmaster")
		print("taskmaster server is killed")

	def emptyline(self):
		pass

	def cmd_loop(self):
		while (42):
			char = read_char()
			self.cmdloop(char)
	

def signal_handler(sig, frame):
    sys.exit(0)

if __name__ == "__main__":
	signal.signal(signal.SIGINT, signal_handler)
	InputInterpretor().cmdloop()
