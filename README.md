# rVsrg

**A high-performance Vertical Scrolling Rhythm Game engine written in Rust**

---

## Features

- **4K gameplay** with fully customizable keybinds
- **Multiple hit window modes** — osu! OD and Etterna Judge support
- **High performance** — Multi-threaded architecture with 200 TPS game logic
- **Skinnable UI** — TOML-based configuration for custom skins
- **Replay system** — Deterministic simulation for accurate score recalculation
- **Practice mode** — Checkpoints to practice difficult sections
- **Variable rate** — Play maps at different speeds (0.5x - 2.0x)
- **Difficulty ratings** — Etterna MSD and osu! SR calculations

## Supported Formats

- **osu!mania** `.osu` files 

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (2024 edition)
- A GPU with Vulkan, Metal, or DX12 support

### Building from Source

```bash
# Clone the repository
git clone https://github.com/your-username/rvsrg.git
cd rvsrg

# Build in release mode (recommended)
cargo build --release

# Run the game
cargo run --release
```

### Adding Beatmaps

1. Create a `songs/` folder in the project root
2. Add osu!mania beatmap folders (each containing `.osu` files)
3. Launch the game — beatmaps are scanned automatically

## Default Controls

### Menu Navigation

| Action | Key |
|--------|-----|
| Navigate | Arrow Keys |
| Confirm | Enter |
| Back | Escape |
| Change Rate | pageup / pagedown |
| Settings | Ctrl+o |
| Practice Mode | F3 |
| Rescan Songs | F8 |

### Gameplay (4K Default)

| Column | Key |
|--------|-----|
| 1 | D |
| 2 | F |
| 3 | J |
| 4 | K |

### Practice Mode

| Action | Key |
|--------|-----|
| Set Checkpoint | Bracket |
| Return to Checkpoint | Bracket Right |

## Configuration

Settings are stored in `settings.toml`:

```toml
master_volume = 0.5
scroll_speed = 500.0
hit_window_mode = "OsuOD"  # or "EtternaJudge"
hit_window_value = 5.0
current_skin = "default"

[keybinds]
4 = ["KeyD", "KeyF", "KeyJ", "KeyK"]
5 = ["KeyD", "KeyF", "Space", "KeyJ", "KeyK"]
6 = ["KeyS", "KeyD", "KeyF", "KeyJ", "KeyK", "KeyL"]
7 = ["KeyS", "KeyD", "KeyF", "Space", "KeyJ", "KeyK", "KeyL"]
```

## Architecture

rVsrg uses a multi-threaded architecture for optimal performance:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Main Thread   │     │  Logic Thread   │     │  Audio Thread   │
│   (Render)      │     │  (200 TPS)      │     │  (Dedicated)    │
├─────────────────┤     ├─────────────────┤     ├─────────────────┤
│ • Window events │────▶│ • Game state    │────▶│ • Audio decode  │
│ • WGPU rendering│◀────│ • Hit detection │◀────│ • Playback sync │
│ • egui UI       │     │ • Score calc    │     │ • Rate control  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

## Project Structure

```
rvsrg/
├── src/
│   ├── main.rs          # Entry point
│   ├── core/            # Core abstractions
│   ├── database/        # SQLite beatmap database
│   ├── difficulty/      # Difficulty calculators
│   ├── input/           # Input handling
│   ├── logic/           # Game logic thread
│   ├── models/          # Data structures
│   ├── render/          # WGPU rendering
│   ├── shaders/         # WGSL shaders
│   ├── shared/          # Cross-thread types
│   ├── states/          # State machines
│   ├── system/          # Thread communication
│   └── views/           # UI components
├── assets/              # Fonts and resources
├── skins/               # Skin configurations
├── songs/               # Beatmap folders
└── .wiki/               # Documentation
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 2024 Edition |
| Graphics | wgpu (WebGPU) |
| Audio | rodio |
| UI | egui |
| Database | SQLite (sqlx) |
| Window | winit |

## Documentation

See the [`.wiki/`](.wiki/) folder for detailed documentation:

- [Home](.wiki/Home.md) — Overview and quick links
- [Architecture](.wiki/Architecture.md) — System design
- [Getting Started](.wiki/Getting-Started.md) — Build instructions
- [Contributing](.wiki/Contributing.md) — How to contribute
- [Keybinds](.wiki/Keybinds.md) — Controls and customization
- [Skinning](.wiki/Skinning.md) — Custom skin creation

## Contributing

Contributions are welcome! Please read the [Contributing Guide](.wiki/Contributing.md) before submitting a PR.

```bash
# Format code
cargo fmt

# Run lints
cargo clippy --all-targets -- -W clippy::all

# Run tests
cargo test
```

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [osu!](https://osu.ppy.sh/) — Beatmap format 
- [Etterna](https://etternaonline.com/) — MSD difficulty calculation
- [wgpu](https://wgpu.rs/) — Cross-platform graphics
- [egui](https://github.com/emilk/egui) — Immediate mode GUI

---