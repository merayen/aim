import argparse
import os

parser = argparse.ArgumentParser(description="Description")
parser.add_argument("--path", nargs="?")
sub_parser = parser.add_subparsers(help="Commands", dest="command")
sub_parser.required = False

opts = parser.parse_args()

if not opts.command:
	opts.command = "run"

if not opts.path:
	opts.path = os.getcwd()

assert os.path.isdir(opts.path), f"Path {opts.path} is not a folder"


if opts.command == "run":
	os.chdir(opts.path)
	assert os.path.isfile("main.py"), f"main.py not found in directory {opts.path}"

	from aim.nodes import load, run
	from aim.ui import run_ui
	from threading import Thread
	with open("main.py") as f:
		# We default with having a UI for our disposal.

		# Create a thread for compiling and running (as a child process) the created program.
		# Kivy needs to run in the mainloop, so we keep these separate.
		class state:
			running = True

		def tick_aim():
			for _ in run(load(f.read())):
				if not state.running:
					break

		aim_thread = Thread(target=tick_aim)
		aim_thread.start()

		try:
			run_ui()
		except KeyboardInterrupt:
			pass

		state.running = False
		aim_thread.join()


# aim
# Automatically runs the project in the current folder using the numpy backend

# aim --current-file file.py --current-line 123
# Not implemented yet.
# Checks for command laid on line 123 in file "file.py" and then tries to process
# the whole file with completion etc.
