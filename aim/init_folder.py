"""
Initializes folder with aim files

This makes the directory ready to be run by aim.
"""
def init_folder(path: str) -> bool:
	import os, sys
	path = os.path.abspath(path)

	if collision_files := {"main.py", ".data", ".local"} & set(os.listdir(path)):
		print(f"Folder is not empty: {' '.join(collision_files)}", file=sys.stderr)
		return False

	os.mkdir(path + os.path.sep + ".data")
	os.mkdir(path + os.path.sep + ".local")

	with open("main.py", "w") as f:
		f.write("out(sine())")

	with open(".data/version", "w") as f:
		f.write("1")

	with open(".gitignore", "w") as f:
		f.write(".local/")

	import subprocess
	git_already_inited = ".git" not in os.listdir()

	if not git_already_inited:
		subprocess.run(["git", "init", "-q"])

	subprocess.run(["git", "add", ".data", ".gitignore", "main.py"])

	if not git_already_inited:
		subprocess.run(["git", "commit", "-qm", "Aim project created"])

	return True
