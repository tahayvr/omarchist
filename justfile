# https://github.com/casey/just

alias u := update
alias b := build
alias c := check

# Build the project
build:
    cargo build

# update the version number (x.y.z | patch | minor | major) for app
update VER:
    ./update-version {{ VER }}
    cargo check

check:
    cargo fmt
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings
