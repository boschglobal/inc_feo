# ASCII Name Experiment

This experiment demonstrates the use of the feo (Fixed Execution Order) framework to print a name in ASCII art with a progressive display approach, where each activity adds one more line to the output.

## Features

- The string as ASCII art is printed in one feo cycle
- The first activity has no inputs but generates the complete ASCII art and outputs it
- Each subsequent activity receives the ASCII art, prints a progressive portion, and forwards it to the next activity
- The height of the ASCII art is 7 lines, with each line printed by a separate activity
- Each activity prints more of the complete ASCII art:
  - Activity 1: Generates the ASCII art (doesn't print)
  - Activity 2: Prints line 0
  - Activity 3: Prints line 1
  - Activity 4: Prints line 2
  - ...and so on until the last activity prints all lines
- Uses a long cycle time (10 seconds by default) to ensure the ASCII art is visible for some time
- Supports both uppercase and lowercase letters and various special characters (.,?!@#$%^&*+-=, etc.)

## How to Build

### For the Host Architecture

Build the project using Cargo:

```bash
# Navigate to the experiment directory
cd /workspaces/vrte-score-integration/demos/ascii_name

# Build the project
cargo build
```

### For ARM64 (aarch64-linux-gnu)

The devcontainer already includes all dependencies for cross-compilation to aarch64-linux-gnu:

```bash
# Navigate to the experiment directory
cd /workspaces/vrte-score-integration/demos/ascii_name

# Build for aarch64-linux-gnu target
cargo build --target aarch64-unknown-linux-gnu
```

## How to Run

Run the ASCII Name display:

```bash
# Run with default settings
cargo run --bin ascii_primary

# Or specify a custom cycle time (in milliseconds)
cargo run --bin ascii_primary 5000  # 5 seconds cycle time
```

To run the aarch64 build, you'll need to either:
1. Transfer the binary to an ARM64 device, or
2. Use QEMU for emulation:

```bash
# Using QEMU to run the ARM64 binary
qemu-aarch64 -L /usr/aarch64-linux-gnu ./target/aarch64-unknown-linux-gnu/debug/ascii_primary
```

## How It Works

1. The program creates a generator activity that produces the complete ASCII art data
2. Each subsequent activity depends on the previous activity, ensuring lines are printed in order
3. Each activity receives the ASCII art from the previous activity, prints its assigned line, and forwards it to the next activity
4. The final result is a cascading display of the ASCII art that grows line by line in a single cycle

## Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                            FEO Application (ascii_primary)                                                           │
├──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                            Primary Agent (ID: 100)                                                                   │
│  ┌───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
│  │                                                         Worker (ID: 40)                                                                           │
│  │                                                                                                                                                   │
│  │  ┌─────────────┐       ┌─────────────┐       ┌─────────────┐       ┌─────────────┐       ┌─────┐       ┌─────────────┐       ┌─────────────┐      │
│  │  │ Activity 0  │[out]  │ Activity 1  │[out]  │ Activity 2  │[out]  │ Activity 3  │[out]  │ ... │[out]  │ Activity 6  │[out]  │ Activity 7  │      │
│  │  │(Generator)  │──────>│(Print Ln 0) │──────>│(Print Ln 1) │──────>│(Print Ln 2) │──────>│     │──────>│(Print Ln 5) │──────>│(Print Ln 6) │      │
│  │  │             │   [in]│             │   [in]│             │   [in]│             │   [in]│     │   [in]│             │   [in]|             |      │
│  │  │Input: None  │       │Input: ASCII │       │Input: ASCII │       │Input: ASCII │       │     │       │Input: ASCII │       │Input: ASCII │      │
│  │  │Out: AsciiArt│       │Out: AsciiArt│       │Out: AsciiArt│       │Out: AsciiArt│       │     │       │Out: AsciiArt│       │Out: None    │      │
│  │  │Depends: -   │       │Depends: 0   │       │Depends: 1   │       │Depends: 2   │       │     │       │Depends: 5   │       │Depends: 6   │      │
│  │  └─────────────┘       └─────────────┘       └─────────────┘       └─────────────┘       └─────┘       └─────────────┘       └─────────────┘      │
│  │                                                                                                                                                   │
│  └───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤
└──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘

Legend:
┌──────┐  Activity       [in]   Input Port       [out]  Output Port
│      │                 ────▶  Data Flow Direction

Data Flow:
- Generator creates complete ASCII art data (7 lines)
- Each Printer activity receives ASCII art, prints its assigned line(s), forwards to next
- Dependencies ensure sequential execution in one FEO cycle
- Communication via Iceoryx2 topics with Linux shared memory backend
```

## Technical Details

- Uses the feo framework for deterministic execution order
- Implements a pipeline of activities with data flow between them
- The ASCII art generator creates visual text representations for letters and special characters
- Communication between activities uses Iceoryx2 (with Linux shared memory)
- Each activity has a unique ID and dependencies on previous activities

## Customization

To modify the displayed text, edit the `DEFAULT_INPUT_STRING` constant in the `config.rs` file:

```rust
// Default input string to generate ASCII art
pub const DEFAULT_INPUT_STRING: &str = "Hello, World!";
```

## Troubleshooting

If you encounter an error about Iceoryx2 topics already existing, try cleaning the temporary files:

```bash
# Clean Iceoryx2 temporary files
rm -rf /tmp/iox*

# Then run the program again
cargo run --bin ascii_primary
```
