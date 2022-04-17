clear
python3 -c '
import sys, tty, termios, os
sys.stdout.write(">")
sys.stdout.flush()
fd = sys.stdin.fileno()
old_settings = termios.tcgetattr(fd)
try:
		tty.setraw(sys.stdin.fileno())
		c = sys.stdin.read(1)
finally:
		termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)

print(c)

if ord(c) in (3,4,7):
	exit()

if c.lower() in "abcdefghijklmnopqrstuvwxyzøæå0123456789 ":
	os.system(f"bash \"run/{c}.sh\"")
else:
	print(f"{c}/{ord(c)}: ¯\_(ツ)_/¯")
'
