use std::{
    fs::File,
    io::{BufWriter, Write},
};

use crossbeam::channel::{Receiver, Sender};

use crate::error::Error;

const FAILED_TO_APPEND_MSG: &str = "failed to append a line to the output file";

pub fn write_entries(
    output_file: &str,
    writer_rx: Receiver<String>,
    writer_quit_tx: Sender<()>,
) -> Result<(), Error> {
    // Recreate the output file and write a line with "#" to mark the start of the playlist
    let out_file = match File::create(shellexpand::tilde(output_file).as_ref()) {
        Ok(it) => it,
        Err(err) => {
            let _ = writer_quit_tx.send(());
            return Err(Error {
                message: format!(
                    "failed to create the output file '{}': {}",
                    output_file, err
                ),
            });
        }
    };

    let mut out_stream = BufWriter::new(out_file);

    if let Err(err) = out_stream.write(b"#\r\n") {
        let _ = writer_quit_tx.send(());
        return Err(Error {
            message: format!("{} '{}': {}", FAILED_TO_APPEND_MSG, output_file, err),
        });
    }

    loop {
        let processed_path = writer_rx.recv().unwrap_or(Default::default());
        if processed_path.is_empty() {
            break;
        }

        if let Err(err) = out_stream.write(format!("{}\r\n", processed_path).as_bytes()) {
            let _ = writer_quit_tx.send(());
            return Err(Error {
                message: format!("{} '{}': {}", FAILED_TO_APPEND_MSG, output_file, err),
            });
        }
    }

    out_stream.flush().unwrap();

    Ok(())
}
