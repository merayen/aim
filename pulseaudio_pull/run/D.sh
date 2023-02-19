# Generate and show documentation in browser
cargo doc &&
bash -c "sleep 1; sensible-browser localhost:9764/$(basename $(pwd))" &
cd target/doc && python3 -m http.server --bind 127.0.0.1 9764
