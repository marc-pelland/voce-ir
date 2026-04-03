# Installation

This guide walks you through installing the Voce IR toolchain.

## Prerequisites

- **Rust 1.85 or later** -- Voce IR uses Rust edition 2024. Install Rust via [rustup](https://rustup.rs/) if you don't have it:

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **A terminal** -- All Voce commands run from the command line.

## Install the CLI

The `voce` binary ships as part of the `voce-validator` crate. Install it with Cargo:

```bash
cargo install voce-validator
```

This compiles and installs the `voce` binary to your Cargo bin directory (typically `~/.cargo/bin/`).

## Verify the installation

```bash
voce --version
```

You should see output like:

```
voce 1.0.0
```

If the command is not found, ensure `~/.cargo/bin` is in your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add that line to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

## Available commands

Run `voce --help` to see all available subcommands:

```
Usage: voce <COMMAND>

Commands:
  validate   Validate a .voce.json IR file
  compile    Compile IR to a target output (HTML, WebGPU, etc.)
  inspect    Print a summary of an IR file
  preview    Compile and open in a browser
  json2bin   Convert JSON IR to binary FlatBuffers format
  bin2json   Convert binary FlatBuffers IR back to JSON
  help       Print this message or the help of the given subcommand(s)
```

## Updating

To update to the latest version:

```bash
cargo install voce-validator --force
```

## Building from source

If you want to work with the development version:

```bash
git clone https://github.com/nicholasgriffintn/voce-ir.git
cd voce-ir
cargo build --workspace
```

The compiled binary will be at `target/debug/voce`.

## Next steps

Now that you have the CLI installed, continue to [Your First IR File](./first-ir.md) to create a minimal Voce IR document by hand.
