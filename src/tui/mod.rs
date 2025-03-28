pub mod core;
pub mod help;

/// generates a string with the requested number of spaces
/// useful for padding text
pub fn space_padding(num: usize) -> String {
    let mut space_str = String::new();
    for _i in 0..num {
        space_str.push(' ');
    }
    space_str
}
