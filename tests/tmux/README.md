# tmux Visual Tests

Automated visual testing for meow slash commands using tmux.

## Prerequisites
- tmux installed
- SQL Server running on localhost:1433 (sa/TestPass123!)
- Rust toolchain

## Run Tests
```bash
chmod +x run_tests.sh update_golden.sh
./run_tests.sh
```

This will:
1. Build meow in release mode
2. Launch it in a tmux session (120×30)
3. Execute each slash command and capture the pane output to `screenshots/`
4. Compare against `golden/` if baselines exist

## Update Golden Baselines
```bash
./run_tests.sh          # Generate fresh screenshots
./update_golden.sh      # Copy screenshots → golden
```

## Manual Testing
```bash
tmux new-session -s meow-manual -x 120 -y 30
./target/release/meow -S localhost,1433 -U sa -P TestPass123! --trust-cert
# Type slash commands and observe output
```
