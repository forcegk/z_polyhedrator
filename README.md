# matrix_rs

`matrix_rs` is a Rust prototype for detecting regular patterns provided via a pattern file on sparse matrices.

## Installation

You will need the latest version of the [rust toolchain](https://rustup.rs) to compile matrix_rs.

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
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### Enabling features
This prototype implements two shortcuts that, depending on the characteristics of the matrix, may reduce search time drastically while producing identical or very similar results. The features are listed on the `Cargo.toml` file and can be activated at compile time as shown below
```bash
cargo build <other_flags> --features shortcut_on_invalidation,shortcut_on_pattern_search
```

## Usage
Debug and release builds are located inside the `target` folder, like below. Needless to say, release builds must be used if good speed is desired, as in this case tend to perform around 20 to 100 times faster.
```bash
# Debug build can be executed with
./target/debug/matrix_rs <flags>

# Release build can be executed with
./target/release/matrix_rs <flags>
```

### Example
If let's say, we wanted to execute `matrix_rs` for `Maragal_1` sparse matrix, the command would be as follows:
```bash
# While on a coding and/or debugging environment
cargo run -- ./data/patterns.txt ./data/sparse/Maragal_1/Maragal_1.mtx

# Looking for performance
./target/release/matrix_rs.exe ./data/patterns.txt ./data/sparse/Maragal_1/Maragal_1.mtx
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Major refactoring is incoming, so do not bother just yet :)

## License
[TO BE DECIDED](https://www.youtube.com/watch?v=SEGLhUZRZdY)