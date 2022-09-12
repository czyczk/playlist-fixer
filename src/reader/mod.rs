use std::{fs::File, io::BufRead, io::BufReader};

use crossbeam::channel::{Receiver, Sender};

use crate::error::Error;

pub fn read_entries(
    input_file: &str,
    task_tx: Sender<String>,
    task_quit_rx: Receiver<()>,
) -> Result<(), Error> {
    let in_file = File::open(shellexpand::tilde(input_file).as_ref()).map_err(|err| Error {
        message: format!("failed to open the input file '{}': {}", input_file, err),
    })?;

    let in_stream = BufReader::new(in_file);

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
