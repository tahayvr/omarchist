# https://github.com/casey/just

alias u := update
alias b := build
alias c := check
alias r := run
alias d := docs

# Run app
run:
    cargo run --release

check:
    cargo fmt
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings

# update the version number (x.y.z | patch | minor | major) for app
update VER:
    ./update-version {{ VER }}
    cargo check

# Build the project
build:
    cargo build --release

docs VER:
    cd docs/ && bun run docs:{{ VER }} 2>/dev/null || true
