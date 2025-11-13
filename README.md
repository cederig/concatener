# concatener

`concatener` is an ultra-fast command-line tool written in Rust that allows you to concatenate multiple files into a single output file.

## Features

`concatener` is designed to be efficient and flexible:

- Concatenate multiple files with space-separated arguments
- Support for wildcard patterns (*.txt, *.log, etc.)
- Directory support - concatenate all files in a directory
- Recursive directory support with -r/--recursive option
- Automatic encoding detection - supports UTF-8, ISO-8859 series, Windows-1252, and more
- Custom output file specification with -o/--output option
- Cross-platform compatibility (Linux, Windows, macOS)
- Built with Rust 2024 edition for optimal performance

## Dependencies

This project uses the following dependencies (as defined in `Cargo.toml`):

- `clap` : Command-line argument parsing with derive macros
- `glob` : Wildcard pattern matching for file selection
- `anyhow` : Error handling and context management
- `encoding_rs` : Automatic character encoding detection and conversion

## Installation

### Prerequisites

Make sure you have Rust and Cargo installed on your system. You can install them by following the instructions on the official Rust website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Compiling for Linux (from Linux)
1. Clone this repository:
    ```sh
    git clone https://github.com/cederig/concatener.git
    cd concatener
    ```
2. Compile the project:
    ```sh
    cargo build --release
    ```
    The executable will be located in `target/release/concatener`.

### Compiling for Windows (from Linux/macOS)

To cross-compile this project for Windows from another operating system (like Linux or macOS), you will need the Rust target for Windows.

1. Add the Windows target to your Rust installation:
    ```sh
    rustup target add x86_64-pc-windows-gnu
    ```

2. Compile the project for the Windows target:
    ```sh
    cargo build --release --target=x86_64-pc-windows-gnu
    ```

The Windows executable will be located in `target/x86_64-pc-windows-gnu/release/concatener.exe`.

### Compiling for macOS (from Linux/macOS)

To cross-compile this project for macOS from another operating system (like Linux or macOS), you will need the Rust target for macOS.

1. Add the macOS target to your Rust installation (choose the correct architecture):
   * For Intel Macs (x86_64):
        ```sh
        rustup target add x86_64-apple-darwin
        ```
   * For Apple Silicon Macs (aarch64):
        ```sh
        rustup target add aarch64-apple-darwin
        ```

2. Compile the project for the macOS target (choose the correct architecture):
   * For Intel Macs:
        ```sh
        cargo build --release --target=x86_64-apple-darwin
        ```
   * For Apple Silicon Macs:
        ```sh
        cargo build --release --target=aarch64-apple-darwin
        ```

The macOS executable will be located in `target/<your_mac_target>/release/concatener`.

## Usage

The basic syntax is as follows:

```sh
./concatener [OPTIONS] <INPUTS>...
```

### Options

- `-o, --output <FILE>` : Output file path (Required)
- `-r, --recursive` : Recursively search directories for files (Optional)
- `<INPUTS>...` : Input files, directories, or patterns to concatenate (Required)

## Examples

### Concatenate specific files
```sh
./concatener -o combined.txt file1.txt file2.txt file3.txt
```

### Concatenate files using wildcard pattern
```sh
./concatener -o all_logs.txt "*.log"
```

### Concatenate all files in a directory
```sh
./concatener -o directory_contents.txt /path/to/directory
```

### Mixed usage with files and patterns
```sh
./concatener -o mixed.txt document.txt "*.md" /path/to/configs/
```

### Concatenate all text files in current directory
```sh
./concatener -o all_text.txt "*.txt"
```

### Recursively concatenate all files in a directory and subdirectories
```sh
./concatener -r -o all_files.txt /path/to/directory
```

### Recursively concatenate files using wildcard patterns
```sh
./concatener -r -o all_rs_files.txt "*.rs"
./concatener -r -o all_txt_files.txt "src/*.txt"
```

**Important**: When using wildcard patterns with the `-r` flag, always use quotes to prevent the shell from expanding the pattern before passing it to the program:

- ✅ **Correct**: `"*.json"` - The program receives the pattern and searches recursively
- ❌ **Incorrect**: `*.json` - The shell expands the pattern, so only files in the current directory are found

### Concatenate files from multiple directories recursively
```sh
./concatener -r -o project_files.txt src/ docs/ tests/
```

### Encoding Detection

`concatener` automatically detects and handles various text file encodings:

- **UTF-8** (with and without BOM)
- **UTF-16LE** (Little-endian, common on Windows)
- **UTF-16BE** (Big-endian, with BOM support)
- **Windows-1252** (common Windows encoding)
- **ISO-8859 series** (European encodings)
- **Asian encodings** (GBK, BIG5, SHIFT_JIS, EUC-JP, EUC-KR)
- **Cyrillic encodings** (KOI8-R, KOI8-U)

The tool automatically tries different encodings in order of likelihood and falls back to UTF-8 with replacement for any undecodable bytes.

Example with mixed encodings:
```sh
./concatener -o mixed_files.txt utf8_file.txt utf16le_file.txt windows1252_file.txt
```

## Tests

This project includes comprehensive unit tests and benchmarks:

```sh
# Run unit tests
cargo test

# Run performance benchmarks
cargo bench

# Run tests with output
cargo test -- --nocapture
```
