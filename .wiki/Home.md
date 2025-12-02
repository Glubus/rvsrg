# rVsrg Wiki

Welcome to the **rVsrg** documentation! This wiki provides comprehensive information for developers and contributors.

## What is rVsrg?

rVsrg (Rust Vertical Scrolling Rhythm Game) is a high-performance rhythm game engine written in Rust. It supports osu!mania-style beatmaps and features:

- 🎮 **4K to 10K gameplay** with customizable keybinds
- 🎵 **Multiple hit window modes** (osu! OD, Etterna Judge)
- 🎨 **Skinnable UI** with TOML configuration
- 📊 **Replay system** with deterministic simulation
- 🎯 **Practice mode** with checkpoints

## Quick Links

- [Architecture Overview](Architecture.md) - System design and thread model
- [Getting Started](Getting-Started.md) - Build and run instructions
- [Contributing](Contributing.md) - How to contribute to the project
- [Keybinds](Keybinds.md) - Default controls and customization
- [Skinning](Skinning.md) - How to create custom skins

## Project Structure

```
rvsrg/
├── src/
│   ├── main.rs          # Entry point
│   ├── core/            # Core input abstractions
│   ├── database/        # SQLite beatmap database
│   ├── difficulty/      # Difficulty calculators (osu!, Etterna)
│   ├── input/           # Input handling and keybind mapping
│   ├── logic/           # Game logic thread
│   ├── models/          # Data structures
│   ├── render/          # WGPU rendering
│   ├── shaders/         # WGSL shaders
│   ├── shared/          # Cross-thread snapshots
│   ├── states/          # Game state machines
│   ├── system/          # Inter-thread communication
│   └── views/           # UI components
├── assets/              # Fonts and resources
├── skins/               # Skin configurations
└── songs/               # Beatmap folders
```

## Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 2024 Edition |
| Graphics | wgpu (WebGPU) |
| Audio | rodio |
| UI | egui |
| Database | SQLite (sqlx) |
| Window | winit |

## License

This project is open source. See the LICENSE file for details.



