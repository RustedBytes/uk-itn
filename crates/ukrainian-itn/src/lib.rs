use std::path::Path;
use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use rustfst::prelude::*;
use rustfst::utils::{acceptor, decode_linear_fst};

#[cfg(feature = "python")]
mod python;

type Grammar = ConstFst<TropicalWeight>;

/// Immutable, thread-safe Ukrainian ITN runtime.
///
/// The grammar files are the OpenFST binaries produced by
/// `python -m ukrainian_itn.export`.
#[derive(Debug)]
pub struct InverseNormalizer {
    tagger: Grammar,
    verbalizer: Grammar,
}

impl InverseNormalizer {
    pub fn from_files(
        tagger_path: impl AsRef<Path>,
        verbalizer_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let tagger = load_grammar(tagger_path.as_ref())?;
        let verbalizer = load_grammar(verbalizer_path.as_ref())?;
        Ok(Self { tagger, verbalizer })
    }

    pub fn normalize(&self, text: &str) -> Result<String> {
        let normalized = normalize_whitespace(text);
        if normalized.is_empty() {
            bail!("input text is empty");
        }

        let prepared = separate_punctuation(&normalized);
        let (canonical, restorations) = canonicalize_orthography(&prepared);
        let mut tagged = rewrite_shortest_path(&self.tagger, &canonical)
            .context("tagger grammar does not accept the input")?;
        tagged = reorder(&tagged);
        restore_word_orthography(&restorations, &mut tagged);

        let verbalized = rewrite_shortest_path(&self.verbalizer, &tagged)
            .context("verbalizer grammar does not accept the input")?;
        Ok(attach_punctuation(verbalized))
    }

    pub fn normalize_or_passthrough(&self, text: &str) -> String {
        self.normalize(text).unwrap_or_else(|_| text.to_owned())
    }
}

fn load_grammar(path: &Path) -> Result<Grammar> {
    let mut fst = VectorFst::<TropicalWeight>::read(path)
        .with_context(|| format!("failed to load FST from {}", path.display()))?;
    if fst.start().is_none() {
        bail!("FST has no start state: {}", path.display());
    }

    for state in fst.states_iter() {
        for tr in fst.get_trs(state)?.trs() {
            if tr.ilabel > 255 || tr.olabel > 255 {
                bail!(
                    "FST contains a non-byte label at state {state}: {}",
                    path.display()
                );
            }
        }
    }

    tr_sort(&mut fst, ILabelCompare {});
    Ok(fst.into())
}

fn rewrite_shortest_path(rule: &Grammar, input: &str) -> Result<String> {
    let labels: Vec<Label> = input.as_bytes().iter().map(|byte| *byte as Label).collect();
    let input_fst: VectorFst<TropicalWeight> = acceptor(&labels, TropicalWeight::one());
    let lattice = rustfst::algorithms::compose::compose::<
        TropicalWeight,
        VectorFst<TropicalWeight>,
        Grammar,
        VectorFst<TropicalWeight>,
        VectorFst<TropicalWeight>,
        &Grammar,
    >(input_fst, rule)?;
    if lattice.start().is_none() {
        bail!("grammar does not accept the input");
    }

    let best: VectorFst<TropicalWeight> = shortest_path(&lattice)?;
    if best.start().is_none() {
        bail!("grammar does not accept the input");
    }
    let path = decode_linear_fst(&best)?;
    let bytes = path
        .olabels
        .into_iter()
        .filter(|label| *label != EPS_LABEL)
        .map(|label| {
            u8::try_from(label)
                .map_err(|_| anyhow!("output label is outside the byte range: {label}"))
        })
        .collect::<Result<Vec<_>>>()?;
    String::from_utf8(bytes).context("grammar produced invalid UTF-8")
}

fn is_python_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'..='\u{000D}'
            | '\u{001C}'..='\u{0020}'
            | '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

fn normalize_whitespace(text: &str) -> String {
    text.split(is_python_whitespace)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

const PUNCTUATION: &[char] = &[',', '.', '!', '?', ';', ':', '(', ')', '«', '»', '…'];

fn separate_punctuation(text: &str) -> String {
    let mut separated = String::with_capacity(text.len() + 16);
    for c in text.chars() {
        if PUNCTUATION.contains(&c) {
            separated.push(' ');
            separated.push(c);
            separated.push(' ');
        } else {
            separated.push(c);
        }
    }
    normalize_whitespace(&separated)
}

fn canonicalize_orthography(text: &str) -> (String, Vec<(String, String)>) {
    let mut canonical_tokens = Vec::new();
    let mut restorations = Vec::new();
    for original in text.split(' ') {
        let canonical = original.replace(['’', 'ʼ'], "'").to_lowercase();
        if canonical != original {
            restorations.push((canonical.clone(), original.to_owned()));
        }
        canonical_tokens.push(canonical);
    }
    (canonical_tokens.join(" "), restorations)
}

fn restore_word_orthography(restorations: &[(String, String)], tagged: &mut String) {
    for (canonical, original) in restorations {
        let canonical_word = format!("word {{ name: \"{canonical}\" }}");
        if let Some(position) = tagged.find(&canonical_word) {
            let original_word = format!("word {{ name: \"{original}\" }}");
            tagged.replace_range(position..position + canonical_word.len(), &original_word);
        }
    }
}

fn reorder_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(r#"(\w+: \".*?\")>> (\w+: \".*\")"#).expect("valid reorder regex")
    })
}

fn reorder(tagged: &str) -> String {
    tagged
        .split("tokens ")
        .map(|chunk| reorder_pattern().replace(chunk, "$2 $1").into_owned())
        .collect::<Vec<_>>()
        .join("tokens ")
}

fn attach_punctuation(mut text: String) -> String {
    for mark in [",", ".", "!", "?", ";", ":", ")", "»", "…"] {
        text = text.replace(&format!(" {mark}"), mark);
    }
    for mark in ["(", "«"] {
        text = text.replace(&format!("{mark} "), mark);
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn byte_identity_fst() -> VectorFst<TropicalWeight> {
        let mut fst = VectorFst::<TropicalWeight>::new();
        let state = fst.add_state();
        fst.set_start(state).unwrap();
        fst.set_final(state, TropicalWeight::one()).unwrap();
        for label in 1..=255 {
            fst.add_tr(state, Tr::new(label, label, TropicalWeight::one(), state))
                .unwrap();
        }
        fst
    }

    fn string_transducer(input: &str, output: &str) -> VectorFst<TropicalWeight> {
        let input_labels: Vec<_> = input.bytes().map(Label::from).collect();
        let output_labels: Vec<_> = output.bytes().map(Label::from).collect();
        rustfst::utils::transducer(&input_labels, &output_labels, TropicalWeight::one())
    }

    #[test]
    fn normalization_helpers_match_python_pipeline() -> Result<()> {
        let temporary = tempdir()?;
        let tagger = temporary.path().join("tagger.fst");
        let verbalizer = temporary.path().join("verbalizer.fst");
        byte_identity_fst().write(&tagger)?;
        byte_identity_fst().write(&verbalizer)?;
        let normalizer = InverseNormalizer::from_files(&tagger, &verbalizer)?;

        let whitespace = [
            '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{001C}', '\u{001D}',
            '\u{001E}', '\u{001F}', '\u{0020}', '\u{0085}', '\u{00A0}', '\u{1680}', '\u{2000}',
            '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}',
            '\u{2008}', '\u{2009}', '\u{200A}', '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}',
            '\u{3000}',
        ];
        for space in whitespace {
            assert_eq!(normalizer.normalize(&format!("a{space}b"))?, "a b");
        }
        assert_eq!(normalizer.normalize(" « a » ")?, "«a»");
        Ok(())
    }

    #[test]
    fn restores_apostrophes_and_case_only_for_words() -> Result<()> {
        let temporary = tempdir()?;
        let tagger = temporary.path().join("tagger.fst");
        let verbalizer = temporary.path().join("verbalizer.fst");

        for variant in ['’', 'ʼ'] {
            let original = format!("p{variant}iat");
            string_transducer("p'iat", "tokens { word { name: \"p'iat\" } }").write(&tagger)?;
            string_transducer(
                &format!("tokens {{ word {{ name: \"{original}\" }} }}"),
                &original,
            )
            .write(&verbalizer)?;
            let normalizer = InverseNormalizer::from_files(&tagger, &verbalizer)?;
            assert_eq!(normalizer.normalize(&original)?, original);
        }

        string_transducer("kyiv", "tokens { word { name: \"kyiv\" } }").write(&tagger)?;
        string_transducer("tokens { word { name: \"KYIV\" } }", "KYIV").write(&verbalizer)?;
        let normalizer = InverseNormalizer::from_files(&tagger, &verbalizer)?;
        assert_eq!(normalizer.normalize("KYIV")?, "KYIV");
        Ok(())
    }

    #[test]
    fn rejects_non_byte_grammars() -> Result<()> {
        let temporary = tempdir()?;
        let invalid = temporary.path().join("invalid.fst");
        let valid = temporary.path().join("valid.fst");
        let mut fst = VectorFst::<TropicalWeight>::new();
        let start = fst.add_state();
        let end = fst.add_state();
        fst.set_start(start)?;
        fst.set_final(end, TropicalWeight::one())?;
        fst.add_tr(
            start,
            Tr::new('x' as Label, 300, TropicalWeight::one(), end),
        )?;
        fst.write(&invalid)?;
        byte_identity_fst().write(&valid)?;

        let error = InverseNormalizer::from_files(invalid, valid).unwrap_err();
        assert!(error.to_string().contains("non-byte label"));
        Ok(())
    }
}
