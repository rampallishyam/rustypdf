# PDF Manipulator (pdfman)

# RustyPDF (rustypdf)
CLI tool to compress and merge PDF files using a Rust backend exposed to Python via PyO3/maturin.

## Features
- `compress`: Reduce embedded image dimensions (PNG) with a scale factor (1-10). Placeholder JPEG handling.
- `merge`: Concatenate multiple PDFs preserving page order.

## Installation (local build)
```bash
pip install maturin
maturin develop  # or: maturin build --release && pip install target/wheels/pdfman-*.whl
```

## CLI Usage
```bash
rustypdf --help

pdfman compress --input input.pdf --output out.pdf --scale 7
rustypdf compress --input input.pdf --output out.pdf --scale 7

pdfman merge --inputs a.pdf b.pdf c.pdf --output merged.pdf
rustypdf merge --inputs a.pdf b.pdf c.pdf --output merged.pdf
```

Scale meaning: 1 = minimal compression, 10 = maximum (down to ~25% linear dimension for PNG images). JPEG currently left unchanged (placeholder logic).

## Python API
```python
from rustypdf import compress_pdf, merge_pdfs
merge_pdfs(["a.pdf", "b.pdf"], "merged.pdf")
compress_pdf("in.pdf", "out.pdf", 5)
```


## Notes / Limitations
- Compression is basic and may not always reduce file size, especially for PDFs with already-compressed JPEGs or complex content.
- `rustypdf compress` will automatically detect Ghostscript and, if missing, will best-effort install it using the platform package manager:
	- Linux: apt/dnf/yum/zypper/pacman (requires appropriate privileges)
	- macOS: Homebrew (`brew`) or MacPorts (`port`)
	- Windows: winget/chocolatey/scoop
	If installation fails or permissions are insufficient, the command falls back to Rust-only compression and prints a note.
- Not all PDFs may be supported; complex structures or encrypted PDFs are out of scope for this minimal example.

## License
MIT
