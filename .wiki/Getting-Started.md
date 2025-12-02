# Getting Started

This guide will help you build and run rVsrg from source.

## Prerequisites

- **Rust** (2024 edition) - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository

### Platform-specific requirements

#### Windows
- Visual Studio Build Tools with C++ workload

#### Linux
- `libasound2-dev` for audio
- `libxkbcommon-dev` for input
- Vulkan drivers for WGPU

#### macOS
- Xcode Command Line Tools

## Building

```bash
# Clone the repository
git clone https://github.com/your-username/rvsrg.git
cd rvsrg

# Build in release mode (recommended for performance)
cargo build --release

# Run the game
cargo run --release
```

## Development Build

For development with faster compile times:

```bash
# Debug build
cargo build

# Run with debug logging
RUST_LOG=debug cargo run
```

## Adding Beatmaps

1. Create a `songs/` folder in the project root
2. Add osu!mania beatmap folders
3. Launch the game - beatmaps are scanned automatically

Supported formats:
- `.osu` files (osu!mania mode)

## Configuration

Settings are stored in `settings.toml`:

```toml
master_volume = 0.5
scroll_speed = 500.0
hit_window_mode = "OsuOD"
hit_window_value = 5.0
current_skin = "default"

[keybinds]
4 = ["KeyD", "KeyF", "KeyJ", "KeyK"]
5 = ["KeyD", "KeyF", "Space", "KeyJ", "KeyK"]
6 = ["KeyS", "KeyD", "KeyF", "KeyJ", "KeyK", "KeyL"]
7 = ["KeyS", "KeyD", "KeyF", "Space", "KeyJ", "KeyK", "KeyL"]
```

## Running Tests

```bash
cargo test
```

## Troubleshooting

### "No audio device found"
Ensure your audio device is properly configured and not in use by another application.

### "WGPU adapter not found"
Update your graphics drivers or try a different backend:
```bash
WGPU_BACKEND=vulkan cargo run --release
# or
WGPU_BACKEND=dx12 cargo run --release
```

### Slow compilation
Use `cargo build` for development and `cargo build --release` only for testing performance.


