use owo_colors::{OwoColorize, Stream};

pub fn success(message: &str) {
    println!(
        "{} {message}",
        "✓".if_supports_color(Stream::Stdout, |text| text.green())
    );
}

pub fn error(message: &str) {
    eprintln!(
        "{} {message}",
        "✗".if_supports_color(Stream::Stderr, |text| text.red())
    );
}

pub fn header(text: &str) {
    println!(
        "{}",
        text.if_supports_color(Stream::Stdout, |text| text.bold())
    );
}

pub fn field(label: &str, value: &str) {
    println!(
        "  {}: {value}",
        label.if_supports_color(Stream::Stdout, |text| text.dimmed())
    );
}
