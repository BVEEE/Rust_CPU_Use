# CPU Use

> Production-ready Rust CLI for CPU usage monitoring and Windows UI automation

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Tech Stack](#tech-stack)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Environment Variables](#environment-variables)
  - [Running Locally](#running-locally)
- [Project Structure](#project-structure)
- [Usage](#usage)
- [API Reference](#api-reference)
- [Testing](#testing)
- [Deployment](#deployment)
- [Roadmap](#roadmap)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

---

## Overview

### What is CPU Use?

CPU Use is a Rust-based command-line tool that combines high-fidelity CPU monitoring with automation helpers for Windows UI Automation (UIA). It targets operators who need live observability of system load, lightweight stress testing, and scripted UI interactions to validate workflows such as Task Manager inspections or application health checks. The tool is designed to stay cross-platform for monitoring while gracefully gating UI automation to Windows.

### Why Does This Exist?

Running performance experiments or validating operational runbooks often requires switching between multiple utilities: one for CPU metrics, another for load generation, and yet another for interacting with UI controls. CPU Use unifies these tasks into a single CLI, reducing context switches and enabling repeatable automation for Windows UIs without sacrificing a smooth terminal experience on other platforms.

### Vision

Over the next 6–12 months, CPU Use aims to deliver:
- A polished TUI with richer charts, trend buffers, and process insights.
- First-class automation playbooks with reusable selectors and fixtures for Windows UI testing.
- Streamlined release artifacts across Windows, macOS, and Linux with reproducible builds and coverage gates baked into CI.

---

## Features

### Core Features

| Feature | Description | Status |
|---------|-------------|--------|
| Live CPU monitor (TUI) | Ratatui-based gauges and tables for per-core and per-process views. | ✅ Complete |
| Stress testing | Multi-threaded CPU load generator for burn-in or benchmarking. | ✅ Complete |
| Windows UI automation | Selector-driven UIA helpers for querying and acting on controls. | 🚧 In Progress |
| Config-driven CLI | TOML and env-based configuration with `RUST_LOG` support. | ✅ Complete |
| Coverage and benchmarks | Tarpaulin coverage gate and Criterion benchmarks for CPU polling. | 🚧 In Progress |

### Feature Details

#### Live CPU Monitoring

A terminal UI built with ratatui and crossterm renders per-core usage gauges and an optional top-process table with periodic refresh. Supports single-snapshot output (`--once`) for scripting or continuous mode for dashboards.

**Capabilities:**
- Per-core and aggregate CPU metrics.
- Optional per-process table filtered to top consumers.
- Keyboard controls (`q`/`Esc`) to exit the TUI safely.

**Limitations:**
- Metrics rely on `sysinfo`; sampling cadence matches the configured interval.
- No built-in remote collection—local host only.

#### Stress Testing

Generates configurable CPU load across threads for a specified duration, useful for smoke tests and capacity checks.

**Capabilities:**
- Thread count and duration flags to tune intensity.
- Runs alongside monitoring for real-time visibility.

**Limitations:**
- CPU-only stress (no memory/disk IO patterns yet).
- Stop conditions are time-based; no adaptive throttling.

#### Windows UI Automation

Selector-based UIA routines to query and act on UI elements (e.g., buttons, text fields) primarily for Windows Task Manager or application validation flows.

**Capabilities:**
- String-based selectors for common UIA attributes.
- Structured `ActionRequest`/`ActionResult` models for automation responses.

**Limitations:**
- Available only on Windows; non-Windows builds return clear errors.
- Requires UI elements to be accessible via UIA.

---

## Tech Stack

### Core Technologies

| Layer | Technology | Version | Purpose |
|-------|------------|---------|---------|
| CLI & TUI | Rust + clap + ratatui + crossterm | Rust stable | Command parsing and terminal UI rendering |
| Metrics | sysinfo | 0.30.x | CPU and process statistics |
| Logging | log + env_logger | 0.11.x | Structured logging driven by `RUST_LOG` |
| Config | config + serde | 0.14.x / 1.0.x | Load settings from files and env |
| Automation | uiautomation (Windows) | 0.2.x | UIA wrappers for selector-based actions |

### Infrastructure

| Component | Technology | Purpose |
|-----------|------------|---------|
| Hosting | Self-contained binary | Local/edge execution without server runtime |
| CI/CD | GitHub Actions | fmt, clippy, tests, coverage, release artifacts |
| Monitoring | Tarpaulin (coverage) + Criterion (bench) | Quality gates and performance baselines |
| Logging | env_logger | Runtime observability via environment configuration |

### AI/Agent Stack (if applicable)

_Not applicable for this project._

---

## Architecture

> For detailed architecture, see [Architecture.md](./docs/Architecture.md) (planned).

### High-Level Overview

```
┌───────────────┐
│    CLI/TUI    │
└───────┬───────┘
        │ clap commands
        ▼
┌───────────────┐     ┌───────────────┐
│ Monitor Flow  │────▶│ sysinfo Poller│
└───────────────┘     └───────────────┘
        │ refresh
        ▼
┌───────────────┐
│ TUI Renderer  │ (ratatui)
└───────────────┘

┌───────────────┐
│ Stress Engine │ (threads)
└───────────────┘

┌──────────────────────────┐
│ UI Automation (Windows)  │
│ uiautomation selectors   │
└──────────────────────────┘
```

### Key Components

1. **CLI Layer** — Clap-driven parser defining `monitor`, `stress`, and `automate` subcommands.
2. **Monitor Loop** — Uses `sysinfo` to poll CPU/process metrics and renders via ratatui.
3. **Stress Engine** — Spawns worker threads to generate CPU load for a configured duration.
4. **UI Automation** — Windows-only helpers that resolve selectors into UIA elements and perform actions.
5. **Config & Logging** — `config` crate for TOML/env ingestion and `env_logger` for diagnostics.

---

## Getting Started

### Prerequisites

Ensure you have the following installed:

| Requirement | Version | Installation |
|-------------|---------|--------------|
| Rust toolchain | stable (see `rust-toolchain.toml`) | [Install Rust](https://www.rust-lang.org/tools/install) |
| Git | any recent | [Install Git](https://git-scm.com/downloads) |
| (Windows UIA) Windows 10+ | — | Required for UI automation features |

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/cpu-use.git
cd cpu-use

# Build workspace
cargo build --release
# Or build just the binary crate
cargo build --release -p cpu-use
```

### Environment Variables

Create a `.env` file in the project root if you want to override defaults:

```bash
# Copy the example environment file if present
cp .env.example .env
```

Configure the following variables:

```env
# Logging
RUST_LOG=info

# CLI configuration
CPU_USE_CONFIG=./cpu-use.toml
```

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `RUST_LOG` | No | `info` | Log verbosity for env_logger |
| `CPU_USE_CONFIG` | No | — | Path to a TOML config file loaded at startup |

### Running Locally

```bash
# Run monitor with per-process table
cargo run -p cpu-use -- monitor --per-process

# Single snapshot for scripting
cargo run -p cpu-use -- monitor --once

# Stress test for 30 seconds across 8 threads
cargo run -p cpu-use -- stress --threads 8 --duration 30
```

The TUI monitor opens in the terminal; press `q` or `Esc` to exit.

### Running with Docker

```bash
# Build the image
docker build -t cpu-use .

# Run the container (monitor example)
docker run --rm -it cpu-use monitor --once
```

---

## Project Structure

```
Rust_CPU_Use/
├── .github/                  # GitHub workflows and templates
│   ├── ISSUE_TEMPLATE/       # Issue templates
│   └── workflows/            # CI/CD pipelines
├── executor/                 # Workspace member containing the cpu-use binary
│   ├── benches/              # Criterion benchmarks
│   ├── src/                  # CLI, monitor, stress, UIA modules
│   └── Cargo.toml            # Crate manifest
├── Cargo.toml                # Workspace manifest
├── rust-toolchain.toml       # Pin to stable toolchain
├── CODE_OF_CONDUCT.md        # Community guidelines
└── README.md                 # Project overview (this file)
```

### Directory Descriptions

| Directory | Purpose |
|-----------|---------|
| `executor/src` | CLI entrypoint, monitor loop, stress harness, selectors, state models, and UI automation helpers. |
| `executor/benches` | Criterion benchmarks for CPU polling and related routines. |
| `.github/workflows` | CI pipelines for fmt, clippy, tests, coverage, and releases. |

---

## Usage

### Basic Usage

```bash
# List commands
cargo run -p cpu-use -- --help

# Start live monitor (default interval 1s)
cargo run -p cpu-use -- monitor --per-process

# Run automation (Windows only)
cargo run -p cpu-use -- automate click 'name="OK" control_type="Button"'
```

### Examples

#### Example 1: Monitor with custom interval

```bash
cpu-use monitor --interval 2 --per-process
```

#### Example 2: Single snapshot for scripting

```bash
cpu-use monitor --once --per-process > cpu.json
```

#### Example 3: Stress test with explicit threads

```bash
cpu-use stress --threads 12 --duration 45
```

### CLI Reference

```bash
cpu-use --help
cpu-use monitor --help
cpu-use stress --help
cpu-use automate --help
```

| Command | Description | Options |
|---------|-------------|---------|
| `monitor` | Render live CPU metrics or emit a single snapshot. | `--interval <secs>`, `--per-process`, `--once` |
| `stress` | Generate CPU load with worker threads. | `--threads <n>`, `--duration <secs>` |
| `automate` | Execute a selector-driven UI action (Windows only). | `--timeout <ms>` plus action args |

---

## API Reference

CPU Use is a CLI tool and does not expose an HTTP API. For automation integration, use the CLI with `--once` to emit structured output or wrap commands in scripts.

---

## Testing

### Running Tests

```bash
# Run unit and integration tests
cargo test --workspace

# Lint
cargo fmt --all
cargo clippy --all-targets -- -D warnings

# Coverage (Linux)
cargo tarpaulin --workspace --timeout 120 --fail-under 80
```

### Test Structure

```
executor/
├── src/         # Module-level unit tests co-located with code
├── benches/     # Criterion benchmarks
└── tests/       # (Add integration tests here)
```

### Coverage Requirements

| Type | Minimum Coverage |
|------|-----------------|
| Lines | 80% |

---

## Deployment

### Environments

| Environment | URL | Branch | Auto-Deploy |
|-------------|-----|--------|-------------|
| CI (build/test) | GitHub Actions | any | ✅ |
| Release artifacts | GitHub Releases | tags (`v*`) | ✅ |

### Deploy Commands

```bash
# Build release binary
cargo build --release -p cpu-use

# Tag and push to trigger release workflow
git tag v1.0.0
git push origin v1.0.0
```

### Pre-Deployment Checklist

- [ ] All tests, fmt, and clippy passing
- [ ] Coverage ≥ 80%
- [ ] Changelog/README updated
- [ ] Version bumped
- [ ] Release artifacts verified locally where applicable

---

## Roadmap

> For detailed roadmap, see `docs/Roadmap.md` (planned).

### Current Phase: Hardening

- [x] TUI monitor with per-core/process support
- [x] Stress harness for load testing
- [ ] Windows UI automation fixtures and tests
- [ ] Extended coverage and benchmark integration in CI

### Upcoming

| Phase | Target Date | Key Deliverables |
|-------|-------------|------------------|
| UIA Stability | Q2 | Mocked Windows automation tests and clearer selector docs |
| Observability | Q3 | Enhanced TUI charts and historical buffers |
| Packaging | Q3 | Release artifacts for Windows/macOS/Linux with checksums |

---

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) (planned) for guidelines.

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Standards

- Follow existing Rust style and run `cargo fmt`/`clippy`
- Add tests for new features
- Update documentation as needed
- Reference acceptance criteria before submitting

---

## License

This project is licensed under the Apache-2.0 License - see the [LICENSE](./LICENSE) file for details.

```
Copyright 2024 CPU Use contributors
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

---

## Contact

### Maintainers

| Name | Role | Contact |
|------|------|---------|
| CPU Use Team | Maintainer | https://github.com/your-org |

### Project Links

| Resource | Link |
|----------|------|
| Repository | https://github.com/your-org/cpu-use |
| Issue Tracker | https://github.com/your-org/cpu-use/issues |
| Discussions | https://github.com/your-org/cpu-use/discussions |

### Get Help

- 📖 Documentation (this README)
- 🐛 [Report a Bug](https://github.com/your-org/cpu-use/issues/new?template=bug_report.md)
- ✨ [Request a Feature](https://github.com/your-org/cpu-use/issues/new?template=feature_request.md)

---

## Acknowledgments

- Inspired by open-source Rust CLI and TUI ecosystems.
- Thanks to the `sysinfo`, `ratatui`, and `uiautomation` maintainers for solid foundations.

---

<p align="center">
  Made with ❤️ by the CPU Use Team
</p>

<p align="center">
  <a href="#cpu-use">Back to Top</a>
</p>
