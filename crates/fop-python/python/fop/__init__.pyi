# Copyright (c) COOLJAPAN OU (Team Kitasan)
# SPDX-License-Identifier: Apache-2.0
#
# Type stubs for the fop native extension module.

class FopConverter:
    """FOP converter for XSL-FO to PDF/SVG/text conversion.

    Example::

        import fop

        converter = fop.FopConverter()
        pdf_bytes = converter.convert_to_pdf(fo_xml_string)
        svg_string = converter.convert_to_svg(fo_xml_string)
        converter.convert_file("input.fo", "output.pdf")
    """

    verbose: bool
    """Whether verbose logging is enabled."""

    def __init__(self) -> None:
        """Create a new FOP converter."""
        ...

    def convert_to_pdf(self, fo_xml: str) -> bytes:
        """Convert XSL-FO string to PDF bytes.

        Args:
            fo_xml: XSL-FO document as a string.

        Returns:
            PDF content as bytes.

        Raises:
            RuntimeError: If the FO document is invalid or conversion fails.
        """
        ...

    def convert_to_svg(self, fo_xml: str) -> str:
        """Convert XSL-FO string to SVG string.

        Args:
            fo_xml: XSL-FO document as a string.

        Returns:
            SVG content as a string.

        Raises:
            RuntimeError: If the FO document is invalid or conversion fails.
        """
        ...

    def convert_to_text(self, fo_xml: str) -> str:
        """Convert XSL-FO string to plain text.

        Args:
            fo_xml: XSL-FO document as a string.

        Returns:
            Plain text content as a string.

        Raises:
            RuntimeError: If the FO document is invalid or conversion fails.
        """
        ...

    def convert_file(self, input_path: str, output_path: str) -> None:
        """Convert a file to another file.

        The output format is detected from the output file extension:
        - ``.pdf`` -> PDF
        - ``.svg`` -> SVG
        - ``.txt`` -> Plain text
        - Any other extension defaults to PDF.

        Args:
            input_path: Path to the input XSL-FO file.
            output_path: Path to the output file.

        Raises:
            IOError: If the input file cannot be read or the output cannot be written.
            RuntimeError: If the FO document is invalid or conversion fails.
        """
        ...

    def validate(self, fo_xml: str) -> tuple[bool, int, str | None]:
        """Validate an XSL-FO document.

        Args:
            fo_xml: XSL-FO document as a string.

        Returns:
            A tuple of ``(valid, node_count, error_message)``:
            - ``valid``: Whether the document parsed successfully.
            - ``node_count``: Number of FO nodes found (0 if invalid).
            - ``error_message``: Error description if invalid, ``None`` if valid.
        """
        ...

    def version(self) -> str:
        """Get version information.

        Returns:
            Version string in the form ``"fop-python X.Y.Z"``.
        """
        ...

    def __repr__(self) -> str:
        """Return a string representation of the converter."""
        ...

def convert_to_pdf(fo_xml: str) -> bytes:
    """One-shot conversion: XSL-FO string to PDF bytes.

    Convenience function that creates a temporary ``FopConverter`` internally.

    Args:
        fo_xml: XSL-FO document as a string.

    Returns:
        PDF content as bytes.

    Raises:
        RuntimeError: If the FO document is invalid or conversion fails.
    """
    ...

def convert_to_svg(fo_xml: str) -> str:
    """One-shot conversion: XSL-FO string to SVG string.

    Convenience function that creates a temporary ``FopConverter`` internally.

    Args:
        fo_xml: XSL-FO document as a string.

    Returns:
        SVG content as a string.

    Raises:
        RuntimeError: If the FO document is invalid or conversion fails.
    """
    ...

def version() -> str:
    """Get version information.

    Returns:
        Version string in the form ``"fop-python X.Y.Z"``.
    """
    ...
