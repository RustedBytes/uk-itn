use std::env;
use std::io::{self, BufRead};
use std::path::PathBuf;

use _rust::InverseNormalizer;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    let (tagger, verbalizer) = match arguments.as_slice() {
        [directory] => {
            let directory = PathBuf::from(directory);
            (
                directory.join("ukrainian_itn_tagger.fst"),
                directory.join("ukrainian_itn_verbalizer.fst"),
            )
        }
        [tagger, verbalizer] => (PathBuf::from(tagger), PathBuf::from(verbalizer)),
        _ => {
            eprintln!(
                "usage: ukrainian_itn_cli <grammar_dir> | <tagger.fst> <verbalizer.fst>\n\
                 Grammars are produced by `python -m ukrainian_itn.export`."
            );
            std::process::exit(2);
        }
    };

    let normalizer = InverseNormalizer::from_files(tagger, verbalizer)?;
    let mut failed = false;
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }
        match normalizer.normalize(&line) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("error: could not normalize {line:?}: {error:#}");
                failed = true;
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
    Ok(())
}
