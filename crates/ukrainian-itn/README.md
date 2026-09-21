# ukrainian-itn

Reusable Rust runtime for Ukrainian inverse text normalization, backed by
[`rustfst`](https://crates.io/crates/rustfst).

The crate embeds its OpenFST binary grammars, so users do not need to install
or locate separate grammar files. Python and Pynini are not required at runtime.

The default feature set has no Python dependencies. The optional `python` feature
enables the internal PyO3 extension used by the companion wheel:

```shell
cargo build --features python
```

```rust
use ukrainian_itn::InverseNormalizer;

let normalizer = InverseNormalizer::new()?;
assert_eq!(normalizer.normalize("двадцять дві тисячі сто один")?, "22101");
# Ok::<(), anyhow::Error>(())
```
