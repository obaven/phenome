pub(super) fn split_plain_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
                continue;
            }
            if ch == '"' {
                in_quotes = false;
                continue;
            }
            current.push(ch);
            continue;
        }
        if ch == '"' {
            in_quotes = true;
            continue;
        }
        if ch.is_whitespace() {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            continue;
        }
        current.push(ch);
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}
