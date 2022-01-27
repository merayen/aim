import time
from .app import parse, write, transpile, llvm_compile, run
import sys
import os

time_all = -time.time()

path = os.path.abspath(sys.argv[1])  # TODO merayen does this really work?
folder, filename = os.path.split(path)

time_parsing = -time.time()
nodes = parse(path)
time_parsing += time.time()

time_writing = -time.time()
write(folder + os.path.sep + filename, nodes)
time_writing += time.time()

c_path = folder + os.path.sep + "output.c"

time_transpiling = -time.time()
transpile(c_path, nodes)
time_transpiling += time.time()

time_compiling = -time.time()
bin_path = llvm_compile(c_path)
time_compiling += time.time()

time_all += time.time()
print("Parsing took: %.3fs" % time_parsing)
print("Writing took: %.3fs" % time_writing)
print("Transpiling took: %.3fs" % time_transpiling)
print("Compiling took: %.3fs" % time_compiling)
print("Total: %.3fs" % time_all)
run(bin_path)
