use crate::minimessage_const::serialize;
use std::io::Write;

pub fn println(message: impl AsRef<str>) {
    let msg = message.as_ref();

    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(msg.as_bytes());
    let _ = stdout.write_all(b"\n");
}

pub fn print_styled(message: impl AsRef<str>) {
    let serialized = serialize::<512>(message.as_ref());
    println(serialized.as_str());
}