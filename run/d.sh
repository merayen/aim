RUSTFLAGS="-Awarnings" cargo build
tmux split-window "lldb-14 target/debug/aim"
