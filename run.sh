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

if c == "q":
	exit()
elif c == "r":
	os.system("RUST_BACKTRACE=1 cargo run")
elif c == "t":
	os.system("RUST_BACKTRACE=1 cargo test")
else:
	print(f"{c}/{ord(c)}: ¯\_(ツ)_/¯")
	exit(1)
'
