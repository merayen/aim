from .app import parse, write
import sys
import os

path = os.path.abspath(sys.argv[1])  # TODO merayen does this really work?
folder, filename = os.path.split(path)
write(folder + os.path.sep + "." + filename, parse(path))
