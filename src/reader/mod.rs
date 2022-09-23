use std::{fs::File, io::BufRead, io::BufReader};

use crossbeam::channel::{Receiver, Sender};
use unicode_bom::Bom;

use crate::error::Error;

pub fn read_entries(
    input_file: &str,
    task_tx: Sender<String>,
    task_quit_rx: Receiver<()>,
) -> Result<(), Error> {
    let input_file = shellexpand::tilde(input_file);
    let in_file = File::open(input_file.as_ref()).map_err(|err| Error {
        message: format!("failed to open the input file '{}': {}", input_file, err),
    })?;

    let bom = detect_bom(input_file.as_ref());
    let mut in_stream = BufReader::new(in_file);

    if let Bom::Utf8 = bom {
        // Skip the first 3 bytes if there's a UTF-8 BOM
        in_stream.seek_relative(3).map_err(|err| Error {
            message: format!("failed to read the input file: {}", err),
        })?;
    }

    for line in in_stream.lines() {
        let line = line.map_err(|err| Error {
            message: format!("failed to read the input file: {}", err),
        })?;

        // Ignore if it is an empty line or starts with "#"
        if line.is_empty() || line.starts_with("#") {
            continue;
        }

        if !task_quit_rx.is_empty() {
            break;
        }
        task_tx.send(line).map_err(|err| Error {
            message: format!("failed to send the task to the task processor: {}", err),
        })?;
    }

    if task_quit_rx.is_empty() {
        end_task_tx(&task_tx);
    }

    Ok(())
}

fn end_task_tx(task_tx: &Sender<String>) {
    let _ = task_tx.send(Default::default());
}

fn detect_bom(input_file: &str) -> Bom {
    let mut in_file = File::open(input_file).unwrap();
    Bom::from(&mut in_file)
}
