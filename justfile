# Define build target
#
# NOTE: this can be changed with an environment variable
target := env_var_or_default("TARGET", "release")
build_option := if target != "debug" { "--" + target } else { "--" }

#  Define binary/target [1]
#
# [1] github.com/jakewilliami/configs/blob/ec00da3f/src/build/justfile-rs#L9-L41
project_dir := justfile_dir() + "/"
bin_name := `cargo pkgid | awk -F'/' '{print $NF}' | cut -d'#' -f1`
target_bin := "target" / target / bin_name
doc_file := "target" / "doc" / bin_name / "index.html"

# Build the project and copy and strip the resulting binary to the root project
build: build-core
    cp -f {{target_bin}} {{project_dir}}
    strip {{bin_name}}

# Core build recipe using `cargo`, used by main build recipe
[private]
build-core:
    cargo build {{build_option}}

# Build the project for Windows
[macos]
build-win: rust-target-win
    # https://stackoverflow.com/a/62853319
    cargo build --target x86_64-pc-windows-gnu {{build_option}}

# Build the project for Windows
[linux]
build-win: get-mingw-w64
    # https://stackoverflow.com/a/62853319
    cargo build --target x86_64-pc-windows-gnu {{build_option}}

[private, linux]
get-mingw-w64: rust-target-win
    dpkg -l | grep -qw mingw-w64 || sudo apt install -y mingw-w64

# Install required Rust toolchain for cross-compiling to Windows
[private, unix]
rust-target-win:
    # TODO: figure out requirements on BSD/macOS
    rustup target add x86_64-pc-windows-gnu

# Check project formatting and linting
fmt: clippy
    cargo fmt --all -- --check

[private]
clippy:
  cargo clippy --all --all-targets --all-features -- --deny warnings

# Update project dependencies in Cargo.lock
update:
    cargo update --locked --package {{bin_name}}

# Run tests
test:
  cargo test --all

# Generate doc
doc:
    cargo doc --open

