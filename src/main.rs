use clap::{crate_authors, crate_version, Parser};
use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use std::io::{stdout, Write};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::{thread, time};

#[derive(Parser)]
#[command(name = "tt", author = crate_authors!("\n"), version = crate_version!())]
/// Tea timer!  Count up in seconds.
struct Cli {}

fn main() {
    let _cli = Cli::parse();

    // Async object to control run of programme
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Stdout object for printing
    let stdout = Arc::new(Mutex::new(stdout()));
    let s = stdout.clone();

    // Overall seconds/partial seconds counters; used in combination
    // with std::thread::sleep to measure time spent.  Note that we
    // count up in 10ths of a second for precision.
    let mut partial_seconds_counter: usize = 0;
    let partials_in_second = 10;

    // Handle Ctrl + C:
    // https://rust-cli.github.io/book/in-depth/signals.html
    ctrlc::set_handler(move || {
        // Tell programme to exit
        r.store(false, Ordering::SeqCst);

        // Show cursor before exitting
        let mut stdout = s.lock().unwrap();
        stdout.execute(cursor::Show).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    // Hide cursor at start of programme
    let mut stdout = stdout.lock().unwrap();
    stdout.execute(cursor::Hide).unwrap();

    // Main counting/printing logic!
    // Note that we count up in 100 milliseconds for better Ctrl+C precision/
    // response time.  If we counted up in seconds, and the user pressed Ctrl+C
    // as the second counter was incremented, they would have to wait close to
    // one second before the programme stops.
    while running.load(Ordering::SeqCst) {
        // Check if our partial timer has added up to one second;
        // if so, update stdout with the current expended time.
        if (partial_seconds_counter % partials_in_second) == 0 {
            // Calculate total seconds expended by the programme
            let seconds = partial_seconds_counter / partials_in_second;

            // Write to time stdout:
            // https://stackoverflow.com/a/59890400/12069968
            stdout.queue(cursor::SavePosition).unwrap();
            stdout
                .write_all(format_seconds(seconds).as_bytes())
                .unwrap();
            stdout.queue(cursor::RestorePosition).unwrap();
            stdout.flush().unwrap();
            stdout.queue(cursor::RestorePosition).unwrap();
            stdout
                .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
                .unwrap();
        }

        // Sleep for one-tenth of a seecond
        thread::sleep(time::Duration::from_millis(100));
        partial_seconds_counter += 1;
    }

    // Ensure cursor is shown before exitting
    stdout.execute(cursor::Show).unwrap();
}

fn format_seconds(seconds: usize) -> String {
    let seconds_rem = seconds % 60;
    let minutes_rem = (seconds / 60) % 60;
    let hours_rem = (seconds / 60) / 60;

    format!("{:0>2}:{:0>2}:{:0>2}", hours_rem, minutes_rem, seconds_rem)
}
