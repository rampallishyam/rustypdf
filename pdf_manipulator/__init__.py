"""
pdf_manipulator - A Python package for PDF compression and merging backed by Rust.
"""

__version__ = "0.1.0"

# Import the Rust functions when the package is imported
try:
    from .pdf_manipulator import compress_pdf, merge_pdfs, get_pdf_info
    __all__ = ["compress_pdf", "merge_pdfs", "get_pdf_info"]
except ImportError:
    # During development, the Rust module might not be built yet
    __all__ = []