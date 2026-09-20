# Ukrainian ITN

Fast WFST-based Inverse Text Normalization (ITN) for Ukrainian. The Python package
uses a Rust runtime and ships with compiled grammars, so using it does not require
Pynini, OpenFST, or a C++ toolchain.

Supported semiotic classes: cardinal, ordinal, decimal, fraction, measure, money, date, time,
telephone, electronic (e-mail/URL), century (Roman numerals), number sign (№), ranges
(numeric/time/date), durations & half-quantities, decades, legal references, scores,
versions, IPv4, postal codes, street addresses.
Also supports high-priority spoken abbreviations and time day-period/time-zone forms.
Punctuation-aware (built for ASR output): `"сто гривень, будь ласка!"` -> `₴100, будь ласка!`,
`"нуль шістдесят сім, сто двадцять три, сорок п'ять, шістдесят сім"` -> `0671234567`
Common Ukrainian apostrophes (`'`, `’`, `ʼ`) and uppercase ASR/text input are accepted
without changing the spelling or case of ordinary words.

## Installation

```shell
pip install ukrainian_itn
```

Installing from source also requires a current Rust toolchain. Published wheels include the
compiled extension and grammar files.

## Usage

```python
from ukrainian_itn import normalize

normalize("це трапилося дві тисячі дев'ятнадцятого числа")  # це трапилося 2019-го числа
normalize("мінус п'ять цілих одна десята відсотка")  # -5.1 %
normalize("двадцять дві тисячі сто один")  # 22101
normalize("сьома година двадцять п'ять хвилин")  # 07:25
normalize("МІНУС П’ЯТЬ ГРИВЕНЬ")  # -₴5
normalize("третя година дня за київським часом")  # 15:00 Europe/Kyiv
normalize("підключи ю ес бі та вай фай")  # підключи USB та Wi-Fi
```

The bundled grammars are loaded lazily on the first call and cached for the lifetime of
the process. `normalize` is thread-safe.

### From command line

```shell
echo "це трапилося дві тисячі дев'ятнадцятого числа" | python -m ukrainian_itn
# or, after `pip install ukrainian_itn`:
echo "це трапилося дві тисячі дев'ятнадцятого числа" | ukrainian-itn
```

```
Options:
  -h, --help     Show this help message and exit
  -j, --json     Return result as JSON
  -v, --verbose  Print original input and normalized to compare
  --version      Show version
```

Will return `це трапилося 2019-го числа`. Lines the grammar cannot parse are reported to
stderr and skipped (exit code 1).

### JSON output

For more advanced usage you can get json output

```python
from ukrainian_itn import normalize

normalize("це трапилося дві тисячі дев'ятнадцятого числа", json=True)
# >>> '[{"word": "це"}, {"word": "трапилося"}, {"ordinal": "2019"}, {"word": "числа"}]'
```

The returned string is guaranteed to be valid JSON, including when pass-through tokens
contain quotes, backslashes, control characters, or non-BMP Unicode.

## Rust runtime

The reusable crate uses [rustfst](https://crates.io/crates/rustfst) and can be embedded
in another Rust application without Python or PyO3 at runtime. To regenerate/export
grammars, install the optional grammar tooling first:

```shell
pip install "ukrainian_itn[grammar]"
python -m ukrainian_itn.export grammars_export
cargo test
cargo build --release
echo "двадцять дві тисячі сто один" | ./target/release/ukrainian_itn_cli grammars_export  # 22101
```

The reusable crate lives under `crates/ukrainian-itn`. Its default feature set is
PyO3-free, so another Rust application can depend on it directly:

```toml
[dependencies]
ukrainian-itn = { path = "../itn-uk/crates/ukrainian-itn" }
```

```rust
use ukrainian_itn::InverseNormalizer;

let normalizer = InverseNormalizer::from_files(
    "grammars_export/ukrainian_itn_tagger.fst",
    "grammars_export/ukrainian_itn_verbalizer.fst",
)?;
assert_eq!(normalizer.normalize("двадцять дві тисячі сто один")?, "22101");
# Ok::<(), anyhow::Error>(())
```

Python bindings are available behind the optional `python` Cargo feature. Maturin enables
that feature when building the Python wheel; ordinary Rust builds leave it disabled.
Backend-specific Python types remain internal so the public Python API is
implementation-independent.

## Grammar development

Pynini is only used to edit, test, and recompile the grammar definitions. It is not
imported by the public Python API and is not an installation dependency. Install it with:

```shell
pip install "ukrainian_itn[grammar]"
```

On platforms without a Pynini wheel, OpenFST development headers may also be needed.
The grammar tooling exposes taggers and verbalizers directly. For example:

```python
from ukrainian_itn.wfst import get_normalizer, apply_fst_text

apply_fst_text("мінус п'ять цілих одна десята відсотка", get_normalizer().classify.fst)
```

This returns `tokens { measure { negative: "true" integer_part: "5" fractional_part: "1" units: "%" } }`.

## Development

```shell
uv sync                # install test and grammar-development dependencies
uv run pytest          # run tests
uv run ruff check .    # lint
uv build               # build sdist + wheel
```
