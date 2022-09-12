use std::thread;

use crossbeam::channel::{bounded, unbounded};
use playlist_fixer::{
    conf, error::Error, reader, stats, task_processor, writer, DEFAULT_CONF_PATH,
};

fn main() -> Result<(), Error> {
    let config = conf::load_conf(DEFAULT_CONF_PATH)?;

    let (task_tx, task_rx) = bounded(1024);
    let (task_quit_tx, task_quit_rx) = bounded(1);
    let (stats_tx, stats_rx) = unbounded();
    let (writer_tx, writer_rx) = bounded(1024);
    let (writer_quit_tx, writer_quit_rx) = bounded(1);

    let reader_handle =
        thread::spawn(move || reader::read_entries(&config.input_file, task_tx, task_quit_rx));
    let task_processor_handle = thread::spawn(move || {
        task_processor::process_task(
            &config.new_ext,
            task_rx,
            stats_tx,
            writer_tx,
            task_quit_tx,
            writer_quit_rx,
        )
    });
    let stats_handle = thread::spawn(move || stats::process_stats(stats_rx));
    let writer_handle = thread::spawn(move || {
        writer::write_entries(&config.output_file, writer_rx, writer_quit_tx)
    });

    let reader_result = reader_handle.join().unwrap();
    let task_processor_result = task_processor_handle.join().unwrap();
    let stats_result = stats_handle.join().unwrap();
    let writer_result = writer_handle.join().unwrap();

    let mut is_error = false;
    let mut error_msg_vec = vec![];
    if reader_result.is_err() {
        is_error = true;
        error_msg_vec.push(reader_result.err().unwrap().message);
    }
    if task_processor_result.is_err() {
        is_error = true;
        error_msg_vec.push(task_processor_result.err().unwrap().message);
    }
    if stats_result.is_err() {
        is_error = true;
        error_msg_vec.push(stats_result.err().unwrap().message);
    }
    if writer_result.is_err() {
        is_error = true;
        error_msg_vec.push(writer_result.err().unwrap().message);
    }

    if is_error {
        eprintln!();
        eprintln!("App failed with error(s):");
        for msg in error_msg_vec {
            eprintln!("{}", msg);
        }
    }

    Ok(())
}
