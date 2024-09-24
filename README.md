# LMDB Convert Tool

## Overview

The LMDB Convert Tool is a utility designed to facilitate the conversion of data between different formats using the Lightning Memory-Mapped Database (LMDB). This tool is essential for developers and data engineers who need to efficiently manage and transform large datasets stored in LMDB.

## Features

- **Data Conversion**: Convert data between various formats such as JSON, CSV, and LMDB.
- **High Performance**: Leverages LMDB's high-speed database capabilities for fast data processing.
- **Scalability**: Handles large datasets with ease.
- **User-Friendly**: Simple command-line interface for ease of use.


## Installation

To install the LMDB Convert Tool, ensure you have [Rust](https://www.rust-lang.org/) installed on your system. Then, run the following command:

```sh
cargo install lmdb-convert
```

## Usage
The command is structured with commands. Common argument is `--input` which is the path to the input file. The command is the action to be performed on the input file.

```sh
lmdb-tools --input <input_file> <command> 
```

### Commands

#### Convert

To use the LMDB Convert Tool, execute the following command in your terminal:

```sh
lmdb-tools --input <input_file> convert <output_file> --format <format>
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
