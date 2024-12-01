# LMDB Tool

## Overview

The LMDB Management Tool is a utility designed to facilitate the conversion of data between different formats using the Lightning Memory-Mapped Database (LMDB). This tool is essential for developers and data engineers who need to efficiently manage and transform large datasets stored in LMDB.

This tool is designed to enable `mender` (see [Mender GitHub](https://github.com/mendersoftware/mender)) to convert data between different architectures, either from 32bits to 64bits or vice versa. It has been tested only with LMDB databases created by `mender`. See [README.mender.md](README.mender.md) for more details.

## Features

- **Data Conversion**: Convert data for 32bits to 64bits or vice versa.
- **Low Performance**: This implementation is not optimized for performance, as pages are read and written one at a time. For large datasets, consider using a more efficient implementation.
- **Cross-Platform**: The LMDB Tool is compatible with Windows, macOS, and Linux operating systems.
- **Easy to Use**: The LMDB Tool is designed to be user-friendly and easy to use, with a simple command-line interface.

## Installation

To install the LMDB Tool, ensure you have [Rust](https://www.rust-lang.org/) installed on your system. Then, run the following command:

```sh
cargo install lmdb
```

## Usage
The command is structured with commands. Common argument is `--input` which is the path to the input file. The command is the action to be performed on the input file.

```sh
lmdb --input <input_file> <command> 
```

### Commands

#### Convert

The `convert` command is used to convert data between different architectures, either from 32bits to 64bits or vice versa.

```sh
lmdb --input <input_file> convert <output_file> --format <format>
```

with:
- `<output_file>`: Path to the output file.
- `--format <format>`: Desired output format (e.g., `32`, `64`).


## Contributing

We welcome contributions to the LMDB Convert Tool! If you would like to contribute, please fork the repository and submit a pull request. For major changes, please open an issue first to discuss what you would like to change.

## Acknowledgements

We would like to thank the contributors and the open-source community for their support and contributions to this project.


## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For any questions or issues, please open an issue on GitHub or contact us at contact@sentiensfr.
