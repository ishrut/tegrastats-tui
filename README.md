# tegrastats-tui

This is a simple tui application that parses the output of tegrastats from nvidia jetson nano and displays the data in a tui.
It's tested on my nvidia jetson nano only. The parser wouldn't be able to parse more than 4 CPU cores.
If you are interested in a richer terminal interface and featureful check out: https://github.com/rbonghi/jetson_stats

## Dependencies

- tegrastats
- cargo

## Building

```bash
cargo build
```

## Installation

```bash
cargo install tegrastats-tui
```
