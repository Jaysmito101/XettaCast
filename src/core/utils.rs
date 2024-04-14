use std::backtrace::Backtrace;
use native_dialog::MessageDialog;


fn format_backtrace(trace: &std::backtrace::Backtrace) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Call Stack:"));
    for frame in trace.frames() {
        lines.push(format!("{:?}", frame)); 
    }
    lines.join("\n")
}

pub fn die(title: &str, message: &str) {
    let backtrace = Backtrace::capture();

    let message = format!("{}\nBacktrace:\n{}", message, format_backtrace(&backtrace));

    log::error!("{:?}", message);
    
    MessageDialog::new()
    .set_title(title)
    .set_text(&message)
    .show_alert()
    .expect("Failed to show dialog");



    std::process::exit(1);
}