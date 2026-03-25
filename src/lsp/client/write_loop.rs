use std::io::Write;
use std::process::ChildStdin;
use std::sync::mpsc::{self, Sender};
use std::thread;

pub(super) fn spawn_write_loop(stdin: ChildStdin) -> Sender<String> {
    let (tx, receiver) = mpsc::channel::<String>();
    thread::spawn(move || {
        let mut stdin = stdin;
        while let Ok(message) = receiver.recv() {
            if stdin.write_all(message.as_bytes()).is_err() {
                break;
            }
            if stdin.flush().is_err() {
                break;
            }
        }
    });
    tx
}
