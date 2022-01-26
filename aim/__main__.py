from .app import parse, write, transpile, llvm_compile, run
import sys
import os

path = os.path.abspath(sys.argv[1])  # TODO merayen does this really work?
folder, filename = os.path.split(path)
nodes = parse(path)
write(folder + os.path.sep + "." + filename, nodes)

c_path = folder + os.path.sep + "output.c"
transpile(c_path, nodes)
bin_path = llvm_compile(c_path)
run(bin_path)
