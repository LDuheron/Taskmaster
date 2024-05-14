# module pour creer des cli plus facilement, definit les commandes que lutilisateur peut saisir et les reponses corespondantes
# # gerer les erreurs de commande avec methode default appelee quand lutililisateur entre une commande invalide
import cmd

#pour lire char par char
import sys, tty, termios

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


# start stop restart status

class InputInterpretor(cmd.Cmd):
	prompt = 'Taskmaster > '

	def do_start(self, arg):
		#checker si l'arg corespond au config file
		print("Restart command :) ")

	def cmd_loop(self):
		while (42):
			char = read_char()
			if (char == '\x1b[A'):
				continue
			else:
				self.cmdloop(char)

if __name__ == "__main__":
	InputInterpretor().cmdloop()
