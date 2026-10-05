# VirtualDevice

Rust library for board input/output memory images. It supports TwinCAT 2, TwinCAT 3, and Comizoa board layouts, with digital, analog, and raw byte access.

## Build and run

```bash
cargo check --all-targets
cargo test
cargo run
```

## Board memory API

Create a typed `IoAddress` and use the matching `BoardMemory` methods:

```rust
use virtualdevice::{AccessKind, AmsAddr, BoardMemory, Direction, IoAddress, TwinCAT3};

let output = IoAddress::<TwinCAT3>::new(
    Direction::Output,
    0,
    0,
    AccessKind::Digital { bit_position: 0 },
);
let mut board = BoardMemory::<TwinCAT3>::new(AmsAddr::default(), 1, 1);
board.write_digital_output(&output, true)?;
assert!(board.read_digital_output(&output)?);
# Ok::<(), &'static str>(())
```

Digital input/output methods are direction-specific. Analog values use little-endian byte representation, and raw byte methods access the selected input or output image.

## Benchmark

Run board memory benchmarks in release mode:

```bash
cargo bench --bench board_memory
```
