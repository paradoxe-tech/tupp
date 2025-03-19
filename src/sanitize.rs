pub fn trim_extra_spaces(input: &str) -> String {
    let trimmed = input.trim();
    let mut result = String::new();
    let mut prev_char = ' ';

    for c in trimmed.chars() {
        if !c.is_whitespace() || !prev_char.is_whitespace() {
            result.push(c);
        }
        prev_char = c;
    }

    result
}