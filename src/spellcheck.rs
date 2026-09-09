use std::collections::HashSet;
use std::fs;
use std::sync::OnceLock;

static EN: OnceLock<HashSet<String>> = OnceLock::new();
static VI: OnceLock<HashSet<String>> = OnceLock::new();

fn load(path: &str) -> HashSet<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

fn en() -> &'static HashSet<String> {
    EN.get_or_init(|| load("dictionaries/en_words.txt"))
}

fn vi() -> &'static HashSet<String> {
    VI.get_or_init(|| load("dictionaries/vi_words.txt"))
}

fn checkers(lang: &str) -> Vec<&'static HashSet<String>> {
    match lang {
        "en" => vec![en()],
        "vi" => vec![vi()],
        _ => vec![en(), vi()],
    }
}

pub fn spellcheck(token: &str, whitelist: &HashSet<String>, lang: &str) -> bool {
    if whitelist.contains(token) {
        return false;
    }
    let lower = token.to_lowercase();
    let sets = checkers(lang);
    !sets.iter().any(|s| s.contains(token) || s.contains(&lower))
}
