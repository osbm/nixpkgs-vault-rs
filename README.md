# 🗄️ nixpkgs-vault

> A comprehensive Nixpkgs package explorer and documentation generator

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Nix](https://img.shields.io/badge/NIX-5277C3.svg?style=flat&logo=NixOS&logoColor=white)](https://nixos.org/)

nixpkgs-vault is a high-performance Rust tool that generates comprehensive documentation and analysis for Nixpkgs packages. It creates an Obsidian-ready knowledge vault with detailed package information, dependencies, and metadata.


## 🚀 Quick Start

### Installation

#### Using Nix (Recommended)

```bash
# Run directly with nix
nix run github:osbm/nixpkgs-vault

# Or install to your profile
nix profile install github:osbm/nixpkgs-vault
```

#### From Source

```bash
git clone https://github.com/osbm/nixpkgs-vault.git
cd nixpkgs-vault
cargo build --release
./target/release/nixpkgs-vault --help
```

### Basic Usage

```bash
# Generate vault for nixos-unstable (default)
nixpkgs-vault

# Use specific revision
nixpkgs-vault --revision nixos-23.11

# Custom output directory
nixpkgs-vault --outdir my-nixpkgs-vault

# Limit processing for testing
nixpkgs-vault --limit 100

# Use more threads for faster processing
nixpkgs-vault --threads 16
```

## 📋 Command Line Options

```
Usage: nixpkgs-vault [OPTIONS]

Options:
  -o, --outdir <OUTDIR>      Output directory [default: nixpkgs-vault]
  -r, --revision <REVISION>  Nixpkgs git revision [default: nixos-unstable]
  -g, --git-url <GIT_URL>    Nixpkgs git url [default: https://github.com/NixOS/nixpkgs.git]
  -j, --threads <THREADS>    Number of parallel threads (0 = auto-detect) [default: 0]
  -l, --limit <LIMIT>        Limit number of packages to process (0 = no limit) [default: 0]
  -h, --help                 Print help
  -V, --version              Print version
```

## 📁 Output Structure

```
nixpkgs-vault/
├── README.md                    # Project overview (from template)
├── packages.json                # Raw package metadata
├── packages/                    # Individual package documentation
│   ├── abc123-firefox-118.0.md
│   ├── def456-python3-3.11.md
│   └── ...
└── .obsidian/                   # Obsidian configuration (from template)
    ├── app.json
    ├── workspace.json
    └── ...
```

## 📝 Package Documentation Format

Each package gets a detailed markdown file with:

- **📋 Package Information**: Name, version, availability, license
- **📝 Description**: Long and short descriptions
- **👥 Maintainers**: GitHub usernames with automatic linking
- **🔧 Build Information**: Derivation paths, outputs, source positions
- **🔗 Dependencies**: Cross-linked dependencies as Obsidian links
- **📁 Input Sources**: Source file paths
- **🏷️ Tags**: Automatic tagging for licenses, maintainers, outputs

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [NixOS](https://nixos.org/) community for the amazing package manager
- [Obsidian](https://obsidian.md/) for the knowledge management inspiration
- Rust community for the excellent ecosystem

*Built with ❤️ using Rust and Nix*
