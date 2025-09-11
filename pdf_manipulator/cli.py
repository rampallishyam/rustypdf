#!/usr/bin/env python3
"""
CLI interface for PDF manipulator tool.
"""

import argparse
import sys
import os
from typing import List


def setup_parser() -> argparse.ArgumentParser:
    """Set up the command line argument parser."""
    parser = argparse.ArgumentParser(
        prog="pdf-manipulator",
        description="A fast PDF compression and merging tool backed by Rust",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Compress a PDF with medium quality (scale 5)
  pdf-manipulator compress --input document.pdf --output compressed.pdf --scale 5
  
  # Merge multiple PDFs
  pdf-manipulator merge --inputs file1.pdf file2.pdf file3.pdf --output merged.pdf
  
  # Get PDF information
  pdf-manipulator info --input document.pdf
        """.strip()
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Compress command
    compress_parser = subparsers.add_parser(
        "compress", 
        help="Compress a PDF by reducing image quality/resolution"
    )
    compress_parser.add_argument(
        "--input", "-i", 
        required=True, 
        help="Input PDF file path"
    )
    compress_parser.add_argument(
        "--output", "-o", 
        required=True, 
        help="Output PDF file path"
    )
    compress_parser.add_argument(
        "--scale", "-s", 
        type=int, 
        default=5, 
        choices=range(1, 11),
        help="Compression scale (1=highest compression, 10=least compression, default=5)"
    )
    
    # Merge command
    merge_parser = subparsers.add_parser(
        "merge", 
        help="Merge multiple PDF files into one"
    )
    merge_parser.add_argument(
        "--inputs", "-i", 
        nargs="+", 
        required=True, 
        help="Input PDF files to merge"
    )
    merge_parser.add_argument(
        "--output", "-o", 
        required=True, 
        help="Output merged PDF file path"
    )
    
    # Info command
    info_parser = subparsers.add_parser(
        "info",
        help="Get information about a PDF file"
    )
    info_parser.add_argument(
        "--input", "-i",
        required=True,
        help="Input PDF file path"
    )
    
    return parser


def validate_input_files(file_paths: List[str]) -> None:
    """Validate that input files exist and are readable."""
    for file_path in file_paths:
        if not os.path.exists(file_path):
            print(f"Error: Input file does not exist: {file_path}", file=sys.stderr)
            sys.exit(1)
        if not os.access(file_path, os.R_OK):
            print(f"Error: Cannot read input file: {file_path}", file=sys.stderr)
            sys.exit(1)
        if not file_path.lower().endswith('.pdf'):
            print(f"Warning: File may not be a PDF: {file_path}", file=sys.stderr)


def validate_output_path(output_path: str) -> None:
    """Validate that output path is writable."""
    output_dir = os.path.dirname(output_path) or "."
    if not os.path.exists(output_dir):
        print(f"Error: Output directory does not exist: {output_dir}", file=sys.stderr)
        sys.exit(1)
    if not os.access(output_dir, os.W_OK):
        print(f"Error: Cannot write to output directory: {output_dir}", file=sys.stderr)
        sys.exit(1)


def handle_compress(args) -> None:
    """Handle the compress command."""
    try:
        from pdf_manipulator import compress_pdf
    except ImportError:
        print("Error: PDF manipulator module not available. Please install the package properly.", file=sys.stderr)
        sys.exit(1)
    
    validate_input_files([args.input])
    validate_output_path(args.output)
    
    print(f"Compressing {args.input} -> {args.output} (scale: {args.scale})")
    
    try:
        compress_pdf(args.input, args.output, args.scale)
        print("✓ Compression completed successfully!")
    except Exception as e:
        print(f"Error during compression: {e}", file=sys.stderr)
        sys.exit(1)


def handle_merge(args) -> None:
    """Handle the merge command."""
    try:
        from pdf_manipulator import merge_pdfs
    except ImportError:
        print("Error: PDF manipulator module not available. Please install the package properly.", file=sys.stderr)
        sys.exit(1)
    
    validate_input_files(args.inputs)
    validate_output_path(args.output)
    
    print(f"Merging {len(args.inputs)} files -> {args.output}")
    print("Input files:")
    for i, input_file in enumerate(args.inputs, 1):
        print(f"  {i}. {input_file}")
    
    try:
        merge_pdfs(args.inputs, args.output)
        print("✓ Merge completed successfully!")
    except Exception as e:
        print(f"Error during merge: {e}", file=sys.stderr)
        sys.exit(1)


def handle_info(args) -> None:
    """Handle the info command."""
    try:
        from pdf_manipulator import get_pdf_info
    except ImportError:
        print("Error: PDF manipulator module not available. Please install the package properly.", file=sys.stderr)
        sys.exit(1)
    
    validate_input_files([args.input])
    
    try:
        page_count, version = get_pdf_info(args.input)
        print(f"File: {args.input}")
        print(f"Pages: {page_count}")
        print(f"PDF Version: {version}")
        
        # Get file size
        file_size = os.path.getsize(args.input)
        if file_size < 1024:
            size_str = f"{file_size} bytes"
        elif file_size < 1024 * 1024:
            size_str = f"{file_size / 1024:.1f} KB"
        else:
            size_str = f"{file_size / (1024 * 1024):.1f} MB"
        print(f"File Size: {size_str}")
        
    except Exception as e:
        print(f"Error reading PDF info: {e}", file=sys.stderr)
        sys.exit(1)


def main() -> None:
    """Main CLI entry point."""
    parser = setup_parser()
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        sys.exit(1)
    
    # Handle commands
    if args.command == "compress":
        handle_compress(args)
    elif args.command == "merge":
        handle_merge(args)
    elif args.command == "info":
        handle_info(args)
    else:
        print(f"Unknown command: {args.command}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()