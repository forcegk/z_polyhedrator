# matrix_rs

`matrix_rs` is a Rust prototype for detecting regular patterns provided via a pattern file on sparse matrices.

## Installation

You will need the latest version of the [rust toolchain](https://rustup.rs) to compile matrix_rs. To use the python utils you will also need [python3(.11.2)](https://www.python.org/downloads/) installed and in the $PATH, along with all packages from `utils/requirements.txt` installed.

### Debug build
To build the debug version the next command can be run
```bash
cargo build
```

However, compiling for debug only does not make any sense unless actual debugging is required. Most of the times, a simple run of the program is desired. To achiveve this, the command below can be used
```bash
cargo run -- <flags>
```

### Optimized build
To make a release build you can just run
```bash
cargo build --release
```

Optionally, to compile to the C equivalent of `-march=native` compilation must be performed with these flags set:
```bash
RUSTFLAGS=""-C opt-level=3 -C target-cpu=native" cargo build --release
```

## Usage
Debug and release builds are located inside the `target` folder, like below. Needless to say, release builds must be used if good speed is desired, as in this case tend to perform around 20 to 100 times faster.
```bash
# Debug build can be executed with
./target/debug/matrix_rs <flags>

# Release build can be executed with
./target/release/matrix_rs <flags>
```

### Command line options
Help on command line options can be obtained by adding `--help` to the cmdline. A sample (and not necessarilly updated) help output is as below:
```
./target/release/matrix_rs --help
ARGS:
    <patterns_file_path>
      File containing pattern list

    <matrixmarket_file_path>
      Input MatrixMarket file

OPTIONS:
    --print-pattern-list
      Print patterns parsed from pattern list

    --print-ast-list
      Print 1D piece list (AST list) before any dimensionality augmentation

    --print-uwc-list
      Print uwc and distinct uwc lists after dimensionality augmentation

    -ti, --transpose-input
      Transpose matrix at input

    -to, --transpose-output
      Transpose matrix at output

    --search-flags <search_flags>
      [2D SEARCH] Search Flags. Valid options: {[PatternFirst], CellFirst} where [] = default.

    -w, --write-spf <output_spf_file_path>
      Write to custom SPF file. By default writes to matrix_market_file.mtx.spf

    -a, --augment-dimensionality <augment_dimensionality>
      Augment dimensionality

    -pl, --augment-dimensionality-piece-cutoff <augment_dimensionality_piece_cutoff>
      Minimum piece length for dimensionality augmentation

    -psmin, --augment-dimensionality-piece-stride-min <augment_dimensionality_piece_stride_min>
      Min stride for augment dimensionality search

    -psmax, --augment-dimensionality-piece-stride-max <augment_dimensionality_piece_stride_max>
      Max stride for augment dimensionality search

    -h, --help
      Prints help information.
```

### Example
If let's say, we wanted to execute `matrix_rs` for `Maragal_1` sparse matrix, the command would be as follows:
```bash
# While on a coding and/or debugging environment
cargo run -- ./data/patterns.txt ./data/sparse/Maragal_1/Maragal_1.mtx

# Looking for performance
./target/release/matrix_rs.exe ./data/patterns.txt ./data/sparse/Maragal_1/Maragal_1.mtx
```

However, you will notice that these commands produce no output. Some frequent use cases can be:

#### Printing AST list
```bash
./target/release/matrix_rs.exe ./data/patterns.txt ./data/sparse/Maragal_1/Maragal_1.mtx --print-ast-list
```

#### Writing to SPF file
```bash
./target/release/matrix_rs.exe ./data/patterns.txt ./data/sparse/Maragal_1/Maragal_1.mtx -w Maragal_1.spf
```

#### Mixed usage
Needless to say, flags can be combined unless explicitly said. For example, in order to obtain more information about the data transformation process, several flags can be specified at the same time.
```bash
./target/release/matrix_rs.exe ./data/patterns.txt ./data/sparse/Maragal_1/Maragal_1.mtx -w Maragal_1.spf --print-ast-list --print-uwc-list --print-pattern-list
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## License
[TO BE DECIDED](https://www.youtube.com/watch?v=SEGLhUZRZdY)
