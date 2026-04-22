# https://github.com/casey/just

alias v := version
alias b := build
alias c := check
alias t := test
alias r := run
alias d := docs
alias rel := release

# Run app
run:
    cargo run --release

# Run the Checker, Clippy, and Formatter
check:
    cargo fmt
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings

# Run the tests
test:
    cargo nextest run --show-progress only

# update the version number (x.y.z | patch | minor | major) for app
version VER:
    cargo set-version {{ if VER =~ "^(patch|minor|major)$" { "--bump " + VER } else { VER } }}

# Build the project
build:
    cargo build --release

# Run the commands for the docs website (dev | build | preview)
docs DOCS:
    cd docs/ && bun run docs:{{ DOCS }} 

# Tag and push a release using version from Cargo.toml
release:
    #!/usr/bin/env bash
    VER=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/')
    git tag -a "v$VER" -m "Release v$VER"
    git push origin main "v$VER"
