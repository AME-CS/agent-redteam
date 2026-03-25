# Contributing to agent-redteam

Thank you for your interest in contributing!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/agent-redteam.git`
3. Build: `cargo build`
4. Test: `cargo test`
5. Run benchmarks: `cargo run -- benchmark`

## How to Contribute

### Add a New Attack Vector

1. Edit `src/attack_engine.rs`
2. Add a new method `init_your_vector()`
3. Call it from `new()`
4. Add corresponding tests in `tests/`

### Improve the RL Synthesizer

The RL engine is in `src/rl_synthesizer.rs`. You can:
- Add new exploration strategies (e.g., Thompson Sampling)
- Improve the reward function
- Add new mutation strategies

### Add Real API Integration

Currently using mock responses. To add real API integration:
1. Edit `src/session_runner.rs`
2. Implement `real_agent_response()` using `reqwest`
3. Add API key config via environment variables

## Pull Request Process

1. Create a feature branch: `git checkout -b feature/your-feature`
2. Make your changes
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Ensure linter passes: `cargo clippy`
6. Format code: `cargo fmt`
7. Commit: `git commit -m "feat: add your feature"`
8. Push: `git push origin feature/your-feature`
9. Open a Pull Request

## Code Style

- Follow Rust conventions (use `cargo fmt`)
- Document public API (use `///` comments)
- Keep functions small and focused
- Add tests for new functionality

## Reporting Bugs

Open an issue with:
- Steps to reproduce
- Expected behavior
- Actual behavior
- Rust version (`rustc --version`)
- OS version

## Security Vulnerabilities

If you discover a security vulnerability in agent-redteam itself, please email security@yourdomain.com instead of opening a public issue.

## Development Commands

```bash
# Build
cargo build

# Run tests
cargo test

# Run with logs
RUST_LOG=debug cargo run -- run --target benchmark

# Lint
cargo clippy

# Format
cargo fmt

# Release build
cargo build --release
```

## Roadmap

See the [README.md](README.md#roadmap) for our current roadmap.

---

Thank you for making agent-redteam better! 🛡️
