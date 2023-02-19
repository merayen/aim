# Show assembly diff view
for x in target/release/deps/*.s ; do
	cp $x $x.previous
done

RUSTFLAGS="--emit asm" cargo build --release
if [[ `tmux list-panes | wc -l` = '1' ]]; then
	tmux split-window vimdiff target/release/deps/*.s target/release/deps/*.s.previous
fi
