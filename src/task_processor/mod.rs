use std::{ffi::OsStr, path::Path};

use crossbeam::channel::{Receiver, Sender};

use crate::{error::Error, stats::Stat};

pub fn process_task(
    new_ext: &str,
    task_rx: Receiver<String>,
    stats_tx: Sender<Stat>,
    writer_tx: Sender<String>,
    task_quit_tx: Sender<()>,
    writer_quit_rx: Receiver<()>,
) -> Result<(), Error> {
    loop {
        let path = task_rx.recv().unwrap();

        if path.is_empty() {
            break;
        }

        let exists = Path::new(&path).exists();
        let processed_path = if exists {
            // If the path exists, it should be directly passed to the writer
            path
        } else {
            // Otherwise, change the extension to the new one
            let result = get_processed_path(&path, new_ext);

            // Also log the original path
            stats_tx
                .send(Stat::OriginalPathNotFound(path))
                .map_err(|err| Error {
                    message: format!("failed to send stat: {}", err),
                })?;

            result
        };

        // Perform a validation to check if the processed path exists
        // Log it if it doesn't exist
        let processed_path_exists = Path::new(&processed_path).exists();
        if !processed_path_exists {
            stats_tx
                .send(Stat::SubstitutionNotFound(processed_path.clone()))
                .map_err(|err| Error {
                    message: format!("failed to send stat: {}", err),
                })?;
        }

        if !writer_quit_rx.is_empty() {
            let _ = task_quit_tx.send(());
            break;
        }
        writer_tx.send(processed_path).map_err(|err| Error {
            message: format!("failed to send the processed path to the writer: {}", err),
        })?;
    }

    if writer_quit_rx.is_empty() {
        end_txes(&stats_tx, &writer_tx);
    }

    Ok(())
}

fn get_processed_path(path: &str, new_ext: &str) -> String {
    let ext_len = Path::new(&path)
        .extension()
        .and_then(OsStr::to_str)
        .map_or(0, |v| v.len() + 1); // The returned extension name doesn't include "."
    let stem = &path[..path.len() - ext_len];
    format!("{}{}", stem, new_ext)
}

fn end_txes(stats_tx: &Sender<Stat>, writer_tx: &Sender<String>) {
    let _ = stats_tx.send(Stat::End);
    let _ = writer_tx.send(Default::default());
}

#[cfg(test)]
mod tests {
    use super::get_processed_path;

    #[test]
    fn test_get_processed_path() {
        {
            let original_path = "/path/to/file.opus";
            let new_ext = ".m4a";

            let expected = "/path/to/file.m4a";
            assert_eq!(expected, get_processed_path(original_path, new_ext));
        }

        {
            let original_path = "/path/to/file-without-ext";
            let new_ext = ".m4a";

            let expected = "/path/to/file-without-ext.m4a";
            assert_eq!(expected, get_processed_path(original_path, new_ext));
        }

        {
            let original_path = "/全字符路径/Artist - Title.opus";
            let new_ext = ".m4a";

            let expected = "/全字符路径/Artist - Title.m4a";
            assert_eq!(expected, get_processed_path(original_path, new_ext));
        }
    }
}
