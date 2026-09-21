"""Public Python API backed by the bundled native runtime."""

from __future__ import annotations

import os
import threading
from typing import Union

from ukrainian_itn._rust import InverseNormalizer as _NativeInverseNormalizer

PathLike = Union[str, os.PathLike[str]]


class InverseNormalizer:
    """Ukrainian inverse text normalizer using compiled WFST grammars.

    With no arguments, the grammars embedded in the native library are used.
    Both paths may be supplied to load custom grammars exported by the
    optional Pynini grammar-development tooling.
    """

    def __init__(
        self,
        tagger_path: PathLike | None = None,
        verbalizer_path: PathLike | None = None,
    ) -> None:
        if (tagger_path is None) != (verbalizer_path is None):
            raise ValueError("tagger_path and verbalizer_path must be provided together")
        if tagger_path is None:
            self._native = _NativeInverseNormalizer()
        else:
            self._native = _NativeInverseNormalizer(
                os.fspath(tagger_path),
                os.fspath(verbalizer_path),
            )

    def normalize(self, text: str, json: bool = False) -> str:
        """Normalize spoken-form Ukrainian text, optionally as token JSON."""
        return self._native.normalize(text, json=json)

    def normalize_or_passthrough(self, text: str) -> str:
        """Normalize text, returning the original text when no rewrite exists."""
        return self._native.normalize_or_passthrough(text)


_normalizer: InverseNormalizer | None = None
_normalizer_lock = threading.Lock()


def get_normalizer() -> InverseNormalizer:
    """Return the process-wide normalizer, initializing it on first use."""
    global _normalizer
    if _normalizer is None:
        with _normalizer_lock:
            if _normalizer is None:
                _normalizer = InverseNormalizer()
    return _normalizer


def normalize(text: str, json: bool = False) -> str:
    """Apply Ukrainian inverse text normalization to ``text``."""
    return get_normalizer().normalize(text, json=json)
