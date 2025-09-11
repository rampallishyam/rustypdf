import argparse
import os
import platform
import shutil
import subprocess
from typing import Optional, List
from pathlib import Path
from pypdf import PdfWriter
from ._rust import compress_pdf


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="rustypdf",
        description="Compress and merge PDF files using a fast Rust backend.",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    p_compress = sub.add_parser(
        "compress", help="Compress a single PDF by reducing embedded image quality/resolution."
    )
    p_compress.add_argument("--input", "-i", required=True, help="Input PDF path")
    p_compress.add_argument("--output", "-o", required=True, help="Output PDF path")
    p_compress.add_argument(
        "--scale",
        "-s",
        type=int,
        default=5,
        choices=range(1, 11),
        metavar="{1..10}",
        help="Compression scale (1=lowest compression, 10=highest). Default: 5",
    )

    p_merge = sub.add_parser("merge", help="Merge multiple PDFs into a single PDF.")
    p_merge.add_argument(
        "--inputs",
        "-i",
        required=True,
        nargs="+",
        help="List of input PDF paths (two or more)",
    )
    p_merge.add_argument("--output", "-o", required=True, help="Output merged PDF path")

    return parser



def _find_gs_executable() -> Optional[str]:
    """Return path to Ghostscript executable if available on PATH (cross-platform)."""
    # Linux/macOS
    for name in ("gs",):
        path = shutil.which(name)
        if path:
            return path
    # Windows common names
    for name in ("gswin64c", "gswin32c"):
        path = shutil.which(name)
        if path:
            return path
    return None


def _ensure_ghostscript() -> Optional[str]:
    """Best-effort: ensure Ghostscript is installed. Returns executable path or None.

    Tries platform package managers: apt/dnf/yum/zypper/pacman (Linux), brew/port (macOS),
    winget/choco/scoop (Windows). Requires appropriate permissions; falls back silently on failure.
    """
    gs = _find_gs_executable()
    if gs:
        return gs

    system = platform.system().lower()

    def _run(cmd: List[str]) -> bool:
        try:
            subprocess.run(cmd, check=True)
            return True
        except Exception:
            return False

    if system == "linux":
        # Try apt
        if shutil.which("apt-get"):
            env = os.environ.copy()
            env.setdefault("DEBIAN_FRONTEND", "noninteractive")
            if shutil.which("sudo"):
                _run(["sudo", "apt-get", "update"])
                _run(["sudo", "apt-get", "install", "-y", "ghostscript"])
            else:
                _run(["apt-get", "update"])  # may fail without root
                _run(["apt-get", "install", "-y", "ghostscript"])  # may fail without root
        # Try dnf/yum/zypper/pacman
        elif shutil.which("dnf"):
            if shutil.which("sudo"):
                _run(["sudo", "dnf", "install", "-y", "ghostscript"])
            else:
                _run(["dnf", "install", "-y", "ghostscript"])  # may fail without root
        elif shutil.which("yum"):
            if shutil.which("sudo"):
                _run(["sudo", "yum", "install", "-y", "ghostscript"])
            else:
                _run(["yum", "install", "-y", "ghostscript"])  # may fail without root
        elif shutil.which("zypper"):
            if shutil.which("sudo"):
                _run(["sudo", "zypper", "install", "-y", "ghostscript"])
            else:
                _run(["zypper", "install", "-y", "ghostscript"])  # may fail without root
        elif shutil.which("pacman"):
            if shutil.which("sudo"):
                _run(["sudo", "pacman", "-Syu", "--noconfirm", "ghostscript"])
            else:
                _run(["pacman", "-Syu", "--noconfirm", "ghostscript"])  # may fail without root

    elif system == "darwin":  # macOS
        if shutil.which("brew"):
            _run(["brew", "update"])
            _run(["brew", "install", "ghostscript"])
        elif shutil.which("port"):
            if shutil.which("sudo"):
                _run(["sudo", "port", "install", "ghostscript"])
            else:
                _run(["port", "install", "ghostscript"])  # may fail without root

    elif system == "windows":
        # Prefer winget, then choco, then scoop
        if shutil.which("winget"):
            _run(["winget", "install", "-e", "--id", "ArtifexSoftware.Ghostscript", "--accept-package-agreements", "--accept-source-agreements"])  # noqa: E501
        elif shutil.which("choco"):
            _run(["choco", "install", "-y", "ghostscript"])  # might be ghostscript.app too
        elif shutil.which("scoop"):
            _run(["scoop", "install", "ghostscript"])  # may require scoop setup

    # Re-check
    return _find_gs_executable()


def _try_ghostscript(in_pdf: Path, out_pdf: Path) -> bool:
    """Try to compress PDF using Ghostscript. Returns True if successful."""
    gs = _ensure_ghostscript()
    if not gs:
        return False
    tmp_out = out_pdf.with_suffix(".gs.pdf")
    cmd = [
        gs,
        "-sDEVICE=pdfwrite",
        "-dCompatibilityLevel=1.4",
        "-dPDFSETTINGS=/ebook",
        "-dNOPAUSE",
        "-dQUIET",
        "-dBATCH",
        f"-sOutputFile={tmp_out}",
        str(out_pdf),
    ]
    try:
        subprocess.run(cmd, check=True)
        tmp_out.replace(out_pdf)
        return True
    except Exception:
        return False


def main(argv=None):
    parser = _build_parser()
    args = parser.parse_args(argv)

    if args.command == "compress":
        in_path = Path(args.input)
        out_path = Path(args.output)
        if not in_path.exists():
            parser.error(f"input file not found: {in_path}")
        compress_pdf(str(in_path), str(out_path), int(args.scale))
        # Try Ghostscript post-processing
        if _try_ghostscript(in_path, out_path):
            print("[rustypdf] Further compressed with Ghostscript.")
        else:
            print("[rustypdf] Ghostscript not found or failed; only Rust compression applied.")
    elif args.command == "merge":
        if len(args.inputs) < 2:
            parser.error("merge requires at least two --inputs")
        out_path = Path(args.output)
        missing = [p for p in args.inputs if not Path(p).exists()]
        if missing:
            parser.error("input file(s) not found: " + ", ".join(missing))
        # Use pypdf for robust merging
        writer = PdfWriter()
        for p in args.inputs:
            writer.append(str(Path(p)))
        with open(out_path, "wb") as f:
            writer.write(f)
        writer.close()
    else:
        parser.error("Unknown command")


if __name__ == "__main__":  # pragma: no cover
    main()
