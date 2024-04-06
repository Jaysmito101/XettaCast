use std::backtrace::Backtrace;
use native_dialog::MessageDialog;

pub fn die(title: &str, message: &str) {
    let backtrace = Backtrace::capture();

    let message = format!("{}\nBacktrace:\n{:?}", message, backtrace);

    log::error!("{:?}", message);
    
    MessageDialog::new()
    .set_title(title)
    .set_text(&message)
    .show_alert()
    .expect("Failed to show dialog");



    std::process::exit(1);
}