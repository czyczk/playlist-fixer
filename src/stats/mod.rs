use crossbeam::channel::Receiver;

use crate::error::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum Stat {
    OriginalPathNotFound(String),
    SubstitutionNotFound(String),
    End,
}

pub fn process_stats(stats_rx: Receiver<Stat>) -> Result<(), Error> {
    let mut summary_substitution_not_found = vec![];

    loop {
        let stat = match stats_rx.recv() {
            Ok(it) => it,
            Err(_) => return Ok(()),
        };

        match stat {
            Stat::End => break,
            Stat::OriginalPathNotFound(path) => {
                println!("{}", path);
            }
            Stat::SubstitutionNotFound(path) => {
                summary_substitution_not_found.push(path);
            }
        }
    }

    // Warn the user if there's substituted path that doesn't exist
    if !summary_substitution_not_found.is_empty() {
        println!(
            "\nAttention: the following {} substituted paths don't exist.",
            summary_substitution_not_found.len()
        );
        for it in summary_substitution_not_found {
            println!("{}", it);
        }
    }

    Ok(())
}
