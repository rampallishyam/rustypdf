import importlib

# Dynamically import the PyO3 native module (built by maturin)
_rustypdf = importlib.import_module("_rustypdf")

compress_pdf = _rustypdf.compress_pdf
merge_pdfs = _rustypdf.merge_pdfs

__all__ = ["compress_pdf", "merge_pdfs"]
