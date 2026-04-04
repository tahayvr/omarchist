# https://github.com/casey/just

alias u := update
alias b := build
alias c := check
alias r := run
alias d := docs
alias rel := release

# Run app
run:
    cargo run || true

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

# Tag and push a release (x.y.z | patch | minor | major)
release VER:
    just update {{ VER }}
    git add Cargo.toml Cargo.lock src/ui/about_page/about_view.rs
    git commit -m "chore(release): {{ VER }}"
    git tag -a "v{{ VER }}" -m "Release v{{ VER }}"
    git push origin main "v{{ VER }}"
