# Copyright (c) COOLJAPAN OU (Team Kitasan)
# SPDX-License-Identifier: Apache-2.0
#
# Python package for fop - XSL-FO to PDF/SVG/Text converter.
# The actual implementation is provided by the native Rust extension module
# built with PyO3 + maturin. This file is intentionally minimal; maturin
# merges the native module into this package automatically.

from .fop import FopConverter, convert_to_pdf, convert_to_svg, version

__all__ = [
    "FopConverter",
    "convert_to_pdf",
    "convert_to_svg",
    "version",
]
