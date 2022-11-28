module_path=$(echo $1 | python3 -c '
import sys
v = sys.stdin.read().strip()
assert v.endswith(".rs")
assert v.startswith("src/")
v = v[len("src/"):-len(".rs")].replace("/", "::")
print(v)
')
ulimit -d 1000000
RUST_BACKTRACE=1 cargo test $module_path
