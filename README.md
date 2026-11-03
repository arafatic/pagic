# Pagic

A fast, lightweight, and open-source PDF editor built with Rust and egui.

## Features
- Clean-room modern UI
- Fast document viewing and editing
- Open-source and privacy-focused

## Distribution status

Pagic is an independent continuation of Recto. Original attribution and third-party licenses
are retained. Release/installer packaging is disabled pending final Pagic artwork and native
installation, association and uninstall validation. No Microsoft Store package is published.

## Getting Started

### Prerequisites
Make sure you have Rust and Cargo installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building & Running
```bash
# Run in development mode
cargo run -p pagic

# Or build for release
cargo build --release -p pagic
```

## License
This project retains the upstream `MIT OR Apache-2.0` license declaration: see the original
[MIT License](LICENSE-MIT) and [Apache License 2.0](LICENSE-APACHE). Dependencies and artwork
retain their own terms; see [verified license sources](licenses/README.md).
