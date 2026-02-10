#!/usr/bin/env python3
"""Convert Rust source files to syntax-highlighted HTML for printing."""

import argparse
import glob
import os
import sys

from pygments import highlight
from pygments.lexers import RustLexer
from pygments.lexers.configs import TOMLLexer as TomlLexer
from pygments.formatters import HtmlFormatter
from pygments.styles import get_all_styles, get_style_by_name

# File patterns and their lexer mappings
RUST_PATTERNS = {
    "*.rs": RustLexer,
    "Cargo.toml": TomlLexer,
    "rust-toolchain.toml": TomlLexer,
    "build.rs": RustLexer,
    "memory.x": None,  # plain text
    "*.ld": None,       # plain text
}

# Patterns for config files in subdirectories
RECURSIVE_EXTRA_PATTERNS = {
    os.path.join(".cargo", "config.toml"): TomlLexer,
}


def get_lexer_for_file(filepath: str):
    """Return the appropriate Pygments lexer for a given file."""
    basename = os.path.basename(filepath)

    for pattern, lexer_cls in RUST_PATTERNS.items():
        if pattern.startswith("*"):
            if basename.endswith(pattern[1:]):
                return lexer_cls() if lexer_cls else None
        elif basename == pattern:
            return lexer_cls() if lexer_cls else None

    for pattern, lexer_cls in RECURSIVE_EXTRA_PATTERNS.items():
        if filepath.replace(os.sep, "/").endswith(pattern.replace(os.sep, "/")):
            return lexer_cls() if lexer_cls else None

    return None


def find_rust_files(directory: str, recursive: bool) -> list[str]:
    """Find Rust-related files in the specified directory."""
    found = set()

    for pattern in RUST_PATTERNS:
        if recursive:
            full = os.path.join(directory, "**", pattern)
        else:
            full = os.path.join(directory, pattern)
        found.update(glob.glob(full, recursive=recursive))

    if recursive:
        for pattern in RECURSIVE_EXTRA_PATTERNS:
            full = os.path.join(directory, "**", pattern)
            found.update(glob.glob(full, recursive=recursive))

    # Exclude build artifacts
    excluded_dirs = {"target", ".git"}
    result = []
    for f in found:
        parts = os.path.relpath(f, directory).split(os.sep)
        if not any(p in excluded_dirs for p in parts):
            result.append(f)

    return sorted(result)


def is_dark_style(style_name: str) -> bool:
    """Detect whether a Pygments style has a dark background."""
    style = get_style_by_name(style_name)
    bg = style.background_color or "#ffffff"
    # Parse hex color and compute perceived brightness
    bg = bg.lstrip("#")
    if len(bg) == 3:
        bg = "".join(c * 2 for c in bg)
    r, g, b = int(bg[0:2], 16), int(bg[2:4], 16), int(bg[4:6], 16)
    luminance = 0.299 * r + 0.587 * g + 0.114 * b
    return luminance < 128


def build_page_css(style_name: str, highlight_css: str) -> str:
    """Generate page CSS adapted to the chosen Pygments style."""
    style = get_style_by_name(style_name)
    code_bg = style.background_color or "#ffffff"

    if is_dark_style(style_name):
        return f"""
    @media print {{
      body {{ margin: 0.5cm; background: #fff !important; color: #000 !important; }}
      .file-section {{ break-inside: avoid; box-shadow: none !important;
                       border: 1px solid #999 !important; }}
      .highlight pre {{ font-size: 9pt; }}
    }}
    body {{ font-family: sans-serif; margin: 2em; background: #2b2b2b; color: #f8f8f2; }}
    h1 {{ color: #f8f8f2; border-bottom: 2px solid #555; padding-bottom: 0.3em; }}
    .file-section {{ background: {code_bg}; border: 1px solid #444; border-radius: 8px;
                     margin-bottom: 2em; padding: 1.5em;
                     box-shadow: 0 2px 4px rgba(0,0,0,0.3); }}
    .file-name {{ font-size: 1.1em; font-weight: bold; color: #66d9ef;
                  background: #333; padding: 0.5em 1em; border-radius: 4px;
                  margin-bottom: 1em; font-family: monospace; }}
    .highlight {{ background: {code_bg}; padding: 1em; border-radius: 4px; overflow-x: auto; }}
    .highlight pre {{ margin: 0; line-height: 1.5; font-size: 0.95em; }}
    {highlight_css}"""
    else:
        return f"""
    @media print {{
      body {{ margin: 0.5cm; }}
      .file-section {{ break-inside: avoid; box-shadow: none !important;
                       border: 1px solid #999 !important; }}
      .highlight pre {{ font-size: 9pt; }}
    }}
    body {{ font-family: sans-serif; margin: 2em; background: #f5f5f5; color: #333; }}
    h1 {{ color: #333; border-bottom: 2px solid #333; padding-bottom: 0.3em; }}
    .file-section {{ background: #fff; border: 1px solid #ddd; border-radius: 8px;
                     margin-bottom: 2em; padding: 1.5em;
                     box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
    .file-name {{ font-size: 1.1em; font-weight: bold; color: #0066cc;
                  background: #e8f0fe; padding: 0.5em 1em; border-radius: 4px;
                  margin-bottom: 1em; font-family: monospace; }}
    .highlight {{ background: {code_bg}; padding: 1em; border-radius: 4px; overflow-x: auto; }}
    .highlight pre {{ margin: 0; line-height: 1.5; font-size: 0.95em; }}
    {highlight_css}"""


def build_html(files: list[str], base_dir: str, style_name: str) -> str:
    """Generate an HTML string from the list of Rust files."""
    default_lexer = RustLexer()
    formatter = HtmlFormatter(style=style_name, nowrap=True)
    highlight_css = HtmlFormatter(style=style_name).get_style_defs(".highlight")

    dir_name = os.path.basename(os.path.normpath(base_dir))
    page_css = build_page_css(style_name, highlight_css)

    sections = ""
    for f in files:
        rel_path = os.path.relpath(f, base_dir)
        with open(f, "r", encoding="utf-8") as fh:
            code = fh.read()

        lexer = get_lexer_for_file(f) or default_lexer
        highlighted = highlight(code, lexer, formatter)
        sections += (
            f'<div class="file-section">\n'
            f'  <div class="file-name">[{rel_path}]</div>\n'
            f'  <div class="highlight"><pre>{highlighted}</pre></div>\n'
            f'</div>\n'
        )

    return f"""<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="utf-8">
  <title>{dir_name} - Rust Source</title>
  <style>{page_css}
  </style>
</head>
<body>
<h1>{dir_name} - Rust Source</h1>
{sections}</body>
</html>
"""


DEFAULT_STYLE = "friendly"


def list_styles():
    """Print all available Pygments styles with dark/light indicator."""
    for name in sorted(get_all_styles()):
        marker = "(dark)" if is_dark_style(name) else "(light)"
        print(f"  {name:24s} {marker}")


def main():
    parser = argparse.ArgumentParser(
        description="Convert Rust source files to syntax-highlighted HTML"
    )
    parser.add_argument("directory", nargs="?", default=None, help="Target directory")
    parser.add_argument("-r", "--recursive", action="store_true",
                        help="Scan subdirectories recursively")
    parser.add_argument("-o", "--output", default=None,
                        help="Output filename (default: <dirname>_rust.html)")
    parser.add_argument("-s", "--style", default=DEFAULT_STYLE,
                        help=f"Pygments color style (default: {DEFAULT_STYLE})")
    parser.add_argument("--list-styles", action="store_true",
                        help="List available styles and exit")
    args = parser.parse_args()

    if args.list_styles:
        print("Available styles:")
        list_styles()
        sys.exit(0)

    if args.directory is None:
        parser.error("directory is required (use --list-styles to see available styles)")

    # Validate style name
    available = list(get_all_styles())
    if args.style not in available:
        print(f"Error: unknown style '{args.style}'", file=sys.stderr)
        print("Use --list-styles to see available styles.", file=sys.stderr)
        sys.exit(1)

    directory = os.path.abspath(args.directory)
    if not os.path.isdir(directory):
        print(f"Error: directory not found: {directory}", file=sys.stderr)
        sys.exit(1)

    files = find_rust_files(directory, args.recursive)
    if not files:
        print("No Rust-related files found.", file=sys.stderr)
        sys.exit(1)

    output_name = args.output or f"{os.path.basename(os.path.normpath(directory))}_rust.html"
    html = build_html(files, directory, args.style)

    with open(output_name, "w", encoding="utf-8") as f:
        f.write(html)

    dark_light = "dark" if is_dark_style(args.style) else "light"
    print(f"Output: {output_name} ({len(files)} files, style: {args.style} [{dark_light}])")
    for rf in files:
        print(f"  - {os.path.relpath(rf, directory)}")


if __name__ == "__main__":
    main()
