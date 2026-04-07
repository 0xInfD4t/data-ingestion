"""
Type stubs for the ``data_ingestion`` native extension module.

Built from ``crates/data-ingestion-python`` via ``maturin``.
"""

from typing import Any, Optional, Union

__version__: str

class ContractEngine:
    """Engine for generating and validating data contracts from raw bytes.

    Create one instance, configure it with the ``set_*`` methods, then call
    :meth:`process_bytes`, :meth:`process_to_format`, :meth:`validate_contract`,
    or :meth:`process_file` for each document.

    Example
    -------
    >>> import data_ingestion
    >>> engine = data_ingestion.ContractEngine()
    >>> engine.set_owner("data-team")
    >>> engine.set_domain("finance")
    >>> engine.set_enrich_pii(True)
    >>> with open("schema.json", "rb") as f:
    ...     contract = engine.process_bytes(f.read(), source_path="schema.json")
    """

    def __init__(self) -> None:
        """Create a new ``ContractEngine`` with default settings.

        Defaults: version ``"1.0.0"``, PII enrichment enabled, nested fields
        included.
        """
        ...

    def set_owner(self, owner: str) -> None:
        """Set the owner identifier for generated contracts."""
        ...

    def set_domain(self, domain: str) -> None:
        """Set the domain label for generated contracts."""
        ...

    def set_version(self, version: str) -> None:
        """Set the semantic version string for generated contracts.

        Default: ``"1.0.0"``.
        """
        ...

    def set_enrich_pii(self, enrich: bool) -> None:
        """Enable or disable automatic PII field detection.

        Default: ``True``.
        """
        ...

    def set_include_nested(self, include: bool) -> None:
        """Enable or disable preservation of nested object fields.

        When ``False``, nested objects are flattened into the parent field list.
        Default: ``True``.
        """
        ...

    def process_bytes(
        self,
        content: bytes,
        format_hint: Optional[str] = None,
        source_path: Optional[str] = None,
    ) -> dict[str, Any]:
        """Process raw bytes into a ``DataContract``, returned as a Python dict.

        Parameters
        ----------
        content:
            Raw file bytes.
        format_hint:
            Optional source format hint: ``"json_schema"``, ``"xsd"``,
            ``"xml"``, ``"csv"``, ``"yaml"``, ``"json"``.
            Pass ``None`` to rely on extension-based detection via
            *source_path*.
        source_path:
            Optional filename used for extension-based detection and lineage
            metadata (e.g. ``"schema.json"``).

        Returns
        -------
        dict
            A Python dictionary representing the generated ``DataContract``.

        Raises
        ------
        ValueError
            If the input cannot be parsed or the format is unsupported.
        RuntimeError
            If the IR → contract transformation fails.
        """
        ...

    def process_to_format(
        self,
        content: bytes,
        format_hint: Optional[str] = None,
        source_path: Optional[str] = None,
        output_format: str = "json",
    ) -> str:
        """Process raw bytes and serialize the contract to the specified format.

        Parameters
        ----------
        content:
            Raw file bytes.
        format_hint:
            Optional source format hint (see :meth:`process_bytes`).
        source_path:
            Optional filename for detection / lineage.
        output_format:
            Target format: ``"json"``, ``"yaml"``, ``"xml"``, or ``"csv"``.

        Returns
        -------
        str
            The serialized contract in the requested format.

        Raises
        ------
        ValueError
            If the input or output format is invalid.
        """
        ...

    def validate_contract(
        self,
        contract: dict[str, Any],
    ) -> dict[str, Any]:
        """Validate a ``DataContract`` provided as a Python dict.

        Parameters
        ----------
        contract:
            A Python dictionary previously returned by :meth:`process_bytes`
            or :meth:`process_file`.

        Returns
        -------
        dict
            ``{"valid": bool, "warnings": list[str], "errors": list[str]}``

        Raises
        ------
        ValueError
            If the dict cannot be deserialized to a valid ``DataContract``.
        """
        ...

    def process_file(
        self,
        path: str,
        format_hint: Optional[str] = None,
        output_format: Optional[str] = None,
    ) -> Union[dict[str, Any], str]:
        """Process a file by path and return the ``DataContract``.

        This method is only available on native (non-WASM) targets.

        Parameters
        ----------
        path:
            Filesystem path to the input file.
        format_hint:
            Optional source format hint (see :meth:`process_bytes`).
        output_format:
            If provided, serialize the contract to this format and return a
            ``str`` instead of a ``dict``.  One of ``"json"``, ``"yaml"``,
            ``"xml"``, ``"csv"``.

        Returns
        -------
        dict or str
            A Python dict when *output_format* is ``None``, otherwise a string
            in the requested format.

        Raises
        ------
        IOError
            If the file cannot be read.
        ValueError
            If the input or output format is invalid.
        """
        ...
