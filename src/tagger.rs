pub fn tag(token: &str) -> Vec<(String, &'static str)> {
    let chars: Vec<char> = token.chars().collect();

    if chars.len() >= 2 {
        let first = chars[0];
        let last = chars[chars.len() - 1];
        if (first == '"' || first == '\'') && first == last {
            let inner: String = chars[1..chars.len() - 1].iter().collect();
            let mut result = vec![(first.to_string(), "PUNCT")];
            result.extend(tag(&inner));
            result.push((last.to_string(), "PUNCT"));
            return result;
        }
    }

    if token.starts_with("http://") || token.starts_with("https://") || token.starts_with("www.") {
        return vec![(token.to_string(), "URL")];
    }

    let domain_part = token.split('/').next().unwrap_or(token);
    let labels: Vec<&str> = domain_part.split('.').collect();
    let is_domain = !domain_part.starts_with('.')
        && !domain_part.ends_with('.')
        && !domain_part.is_empty()
        && labels.len() >= 2
        && labels.iter().all(|l| !l.is_empty() && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        && labels.last().map_or(false, |tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()));
    if is_domain {
        return vec![(token.to_string(), "URL")];
    }

    if !token.starts_with('@') && token.matches('@').count() == 1 {
        let parts: Vec<&str> = token.split('@').collect();
        if parts[1].contains('.') {
            return vec![(token.to_string(), "EMAIL")];
        }
    }

    if chars.len() > 1 && (chars[0] == '#' || chars[0] == '@') && chars[1].is_alphanumeric() {
        return vec![(token.to_string(), "HASHTAG")];
    }

    if chars.iter().all(|c| !c.is_alphanumeric()) {
        return vec![(token.to_string(), "PUNCT")];
    }

    if chars.iter().all(|c| c.is_ascii_digit() || matches!(c, ':' | '/' | '-' | '.' | ',' | '%' | '–' | '—' | '~')) {
        return vec![(token.to_string(), "NUMERIC")];
    }

    let time_formats = ["%Hh%M", "%Hg%M", "%H:%M", "%H:%M:%S"];
    if time_formats.iter().any(|f| chrono::NaiveTime::parse_from_str(token, f).is_ok()) {
        return vec![(token.to_string(), "NUMERIC")];
    }

    if let Ok(number) = phonenumber::parse(Some(phonenumber::country::VN), token) {
        if number.is_valid() {
            return vec![(token.to_string(), "PHONE")];
        }
    }

    let units = [
        "kg", "g", "mg", "km", "m", "cm", "mm", "ml", "l", "h", "p", "s", "%", "đ", "vnd", "usd", "s", "h", "min",
    ];
    let digit_end = chars.iter().take_while(|c| c.is_ascii_digit()).count();
    if digit_end > 0 && digit_end < chars.len() {
        let suffix: String = chars[digit_end..].iter().collect::<String>().to_lowercase();
        if units.contains(&suffix.as_str()) {
            return vec![(token.to_string(), "QUANTITY")];
        }
    }

    let mut start = 0;
    while start < chars.len() && !chars[start].is_alphanumeric() {
        start += 1;
    }
    let mut end = chars.len();
    while end > start && !chars[end - 1].is_alphanumeric() {
        end -= 1;
    }

    let mut result = Vec::new();
    if start > 0 {
        result.push((chars[..start].iter().collect(), "PUNCT"));
    }
    if start < end {
        let core: String = chars[start..end].iter().collect();
        if core.contains('/') {
            let parts: Vec<&str> = core.split('/').collect();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    result.push(("/".to_string(), "PUNCT"));
                }
                if part.is_empty() {
                    continue;
                }
                let has_digit = part.chars().any(|c| c.is_ascii_digit());
                let has_alpha = part.chars().any(|c| c.is_alphabetic());
                let part_tag = if has_digit && has_alpha {
                    "CODE"
                } else if has_digit {
                    "NUMERIC"
                } else {
                    "WORD"
                };
                result.push((part.to_string(), part_tag));
            }
        } else {
            let has_digit = core.chars().any(|c| c.is_ascii_digit());
            let has_alpha = core.chars().any(|c| c.is_alphabetic());
            let core_tag = if has_digit && has_alpha {
                "CODE"
            } else if has_digit {
                "NUMERIC"
            } else {
                "WORD"
            };
            result.push((core, core_tag));
        }
    }
    if end < chars.len() {
        result.push((chars[end..].iter().collect(), "PUNCT"));
    }
    result
}