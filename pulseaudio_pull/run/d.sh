# Build and debug
RUSTFLAGS="-Awarnings" cargo build &&
tmux split-window "lldb-14 target/debug/`basename $PWD`"
