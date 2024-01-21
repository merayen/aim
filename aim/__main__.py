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

	from aim.nodes import load
	from aim.run import CompileAndRun

	with open("main.py") as f:
		# We default with having a UI for our disposal.

		# Create a thread for compiling and running (as a child process) the created program.
		compile_and_run = CompileAndRun(load(f.read()))


# aim
# Automatically runs the project in the current folder using the numpy backend

# aim --current-file file.py --current-line 123
# Not implemented yet.
# Checks for command laid on line 123 in file "file.py" and then tries to process
# the whole file with completion etc.
