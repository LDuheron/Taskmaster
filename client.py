import cmd
import sys, tty, termios
import socket
import signal
import sys


HOST = 'localhost'
PORT = 4241

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
		print('Connected')
		sock.send(str.encode('utf-8'))
		sock.close()
	except Exception as e:
		print("Error in send_data:", e)

class InputInterpretor(cmd.Cmd):
	prompt = 'Taskmaster > '

	def	__init__(self):
		super().__init__()

	def	default(self, line):
		print( "Correct format is : [command] [program]\nType 'help' for all commands.")
	
	def do_status(self, arg):
		'Print the status of all the programs\n'
		send_data(f"status {arg}")
		print("Start command :) ")

	def do_start(self, arg):
		'Start the program specified in argument.\n'
		send_data(f"start {arg}")
		print("Start command :) ")
	
	def do_stop(self, arg):
		'Stop the program specified in argument.\n'
		send_data(f"stop {arg}")
		print("Stop command")

	def do_restart(self, arg=None):
		'Restart the program specified in argument.\n'
		send_data(f"restart {arg}")
		print("Restart command")
	
	def do_quit(self, arg):
		'Disconnect the client and quit program.\n'
		print('Client disconnected')
		sys.exit(0)

	def do_reload(self, arg):
		'Reload the config file.\n'
		send_data("reload")
		print('Reloading the config file')

	def cmd_loop(self):
		while (42):
			char = read_char()
			self.cmdloop(char)

def signal_handler(sig, frame):
    sys.exit(0)

if __name__ == "__main__":
	signal.signal(signal.SIGINT, signal_handler)
	InputInterpretor().cmdloop()
