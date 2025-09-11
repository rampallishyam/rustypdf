# PDF Manipulator

A fast Python CLI tool backed by Rust for PDF compression and merging using PyO3/maturin.

## Features

- **compress**: Reduce PDF file size by adjusting image quality/resolution with scale factor (1-10)
- **merge**: Combine multiple PDF files into a single document  
- **info**: Get PDF metadata including page count and version

## Installation

Install directly from the built wheel:

```bash
pip install pdf_manipulator
```

Or build from source:

```bash
# Install maturin if not already installed
pip install maturin

# Build and install in development mode
maturin develop

# Or build wheel
maturin build
pip install target/wheels/pdf_manipulator-*.whl
```

## Usage

### Command Line Interface

The package provides a `pdf-manipulator` command with helpful subcommands:

```bash
# Get help
pdf-manipulator --help

# Compress a PDF (scale 1=highest compression, 10=least compression)
pdf-manipulator compress --input document.pdf --output compressed.pdf --scale 5

# Merge multiple PDFs
pdf-manipulator merge --inputs file1.pdf file2.pdf file3.pdf --output merged.pdf

# Get PDF information
pdf-manipulator info --input document.pdf
```

### Examples

```bash
# High compression (small file size)
pdf-manipulator compress -i large_document.pdf -o small_document.pdf -s 1

# Low compression (preserve quality)  
pdf-manipulator compress -i document.pdf -o compressed.pdf -s 9

# Merge all PDFs in current directory
pdf-manipulator merge -i *.pdf -o merged_document.pdf

# Check PDF details
pdf-manipulator info -i document.pdf
```

## Development

### Building

This project uses PyO3 and maturin to create Python bindings for Rust code:

```bash
# Install development dependencies
pip install maturin

# Build in development mode
maturin develop

# Run tests
python -m pytest  # (if tests are added)

# Build release wheel
maturin build --release
```

### Project Structure

```
pdf-manipulator/
├── src/
│   └── lib.rs              # Rust implementation with PyO3 bindings
├── pdf_manipulator/
│   ├── __init__.py         # Python package
│   └── cli.py              # Command line interface
├── Cargo.toml              # Rust dependencies
├── pyproject.toml          # Python package configuration
└── README.md
```

### Architecture

- **Rust Backend**: Uses `lopdf` crate for PDF manipulation, providing fast native performance
- **Python Bindings**: PyO3 creates seamless Python/Rust integration
- **CLI Interface**: Python argparse provides user-friendly command line interface
- **Error Handling**: Comprehensive error handling for file I/O and PDF parsing errors

## License

MIT License - see LICENSE file for details.

## Technical Details

- Built with Rust for performance-critical PDF operations
- Uses PyO3 for Python/Rust interoperability  
- Leverages `lopdf` crate for PDF parsing and manipulation
- Pip-installable package with entry point configuration
- Cross-platform compatibility

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes and test
4. Submit a pull request

For development setup:
```bash
git clone <repository>
cd pdf-manipulator
maturin develop
```
