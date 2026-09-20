"""WFST-based Inverse Text Normalization (ITN) for Ukrainian.

Public API::

    from ukrainian_itn import normalize

    normalize("двадцять дві тисячі сто один")  # "22101"

The compiled grammars are bundled with the package and evaluated by the native
runtime, so Pynini is not required for normal use.
"""

__version__ = "0.3.0"

__all__ = ["normalize", "InverseNormalizer", "__version__"]


def __getattr__(name):
    if name in ("normalize", "InverseNormalizer"):
        from ukrainian_itn import _api

        return getattr(_api, name)
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")
