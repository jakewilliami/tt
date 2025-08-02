use clap::{Parser, crate_authors, crate_version};
use crossterm::{ExecutableCommand, QueueableCommand, cursor, terminal};
use std::io::{Write, stdout};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::{
    thread,
    time::{Duration, Instant},
};

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

    // Handle Ctrl + C:
    // https://rust-cli.github.io/book/in-depth/signals.html
    ctrlc::set_handler(move || {
        // Tell programme to exit
        r.store(false, Ordering::SeqCst);

        // Show cursor before exiting
        // Use try_lock to avoid deadlock; try to clean up but don't block
        // if mutex is locked
        if let Ok(mut stdout_guard) = s.try_lock() {
            // Leave the final time visible
            let _ = stdout_guard.queue(cursor::Show);
            let _ = stdout_guard.write_all(b"\n");
            let _ = stdout_guard.flush();
        }
    })
    .expect("Error setting Ctrl-C handler");

    // Hide cursor at start of programme
    let mut stdout_guard = stdout.lock().unwrap();
    stdout_guard.execute(cursor::Hide).unwrap();
    drop(stdout_guard);

    // Set start time measurement
    let start_time = Instant::now();

    // Main counting/printing logic!
    //
    // Note that we run the main event loop every 100 milliseconds for better
    // Ctrl+C precision/response time.  If we counted up in seconds, and the
    // user pressed Ctrl+C as the second counter was incremented, they would
    // have to wait to close one second before the programme stops.
    while running.load(Ordering::SeqCst) {
        // Get elapsed time since start
        let elapsed = start_time.elapsed();
        let seconds = elapsed.as_secs();

        // Write to time stdout:
        // https://stackoverflow.com/a/59890400
        let mut stdout_guard = stdout.lock().unwrap();
        stdout_guard
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
        stdout_guard.queue(cursor::SavePosition).unwrap();
        stdout_guard
            .write_all(format_seconds(seconds).as_bytes())
            .unwrap();
        stdout_guard.flush().unwrap();
        stdout_guard.queue(cursor::RestorePosition).unwrap();

        // Release the lock before sleeping
        drop(stdout_guard);

        // Sleep for one-tenth of a second
        thread::sleep(Duration::from_millis(100));
    }

    // Clean up before exiting
    // Leave final time visible (issue #1)
    let mut stdout_guard = stdout.lock().unwrap();
    stdout_guard.queue(cursor::Show).unwrap();
    stdout_guard.write_all(b"\n").unwrap();
    stdout_guard.flush().unwrap();
}

fn format_seconds(seconds: u64) -> String {
    let seconds_rem = seconds % 60;
    let minutes_rem = (seconds / 60) % 60;
    let hours_rem = (seconds / 60) / 60;

    format!("{hours_rem:0>2}:{minutes_rem:0>2}:{seconds_rem:0>2}")
}
