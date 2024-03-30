
use color_print::cprintln;

#[macro_export]
macro_rules! warn {
    ($tokens: tt) => logger(LogLevel::Warning, $tokens)
}

pub enum LogLevel {
    Warning,
    Error,
}

pub fn logger(level: LogLevel, text: String) {
    match level {
        LogLevel::Warning => cprintln!("<y>warning:</> {text}"),
        LogLevel::Error   => cprintln!("<r>error:</> {text}"),
    }
}



