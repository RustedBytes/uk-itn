use std::env;
use std::io::{self, BufRead};
use std::path::PathBuf;

use ukrainian_itn::InverseNormalizer;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    let normalizer = match arguments.as_slice() {
        [] => InverseNormalizer::new()?,
        [directory] => {
            let directory = PathBuf::from(directory);
            InverseNormalizer::from_files(
                directory.join("ukrainian_itn_tagger.fst"),
                directory.join("ukrainian_itn_verbalizer.fst"),
            )?
        }
        [tagger, verbalizer] => {
            InverseNormalizer::from_files(PathBuf::from(tagger), PathBuf::from(verbalizer))?
        }
        _ => {
            eprintln!(
                "usage: ukrainian_itn_cli [<grammar_dir> | <tagger.fst> <verbalizer.fst>]\n\
                 With no arguments, the embedded grammars are used."
            );
            std::process::exit(2);
        }
    };
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
