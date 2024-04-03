use std::{backtrace::Backtrace, thread::sleep};
use native_dialog::{FileDialog, MessageDialog};

pub fn die(title: &str, message: &str) {
    let backtrace = Backtrace::capture();

    let message = format!("{}\nBacktrace:\n{:?}", message, backtrace);

    let configm = MessageDialog::new()
    .set_title(title)
    .set_text(&message)
    .show_alert();

    log::error!("{}", message);

    std::process::exit(1);
}