use hunspell::Hunspell;
use std::collections::HashSet;
use std::sync::OnceLock;

struct HunspellHandle(Hunspell);
unsafe impl Sync for HunspellHandle {}
unsafe impl Send for HunspellHandle {}

static EN: OnceLock<HunspellHandle> = OnceLock::new();
static VI: OnceLock<HunspellHandle> = OnceLock::new();

fn en() -> &'static Hunspell {
    &EN.get_or_init(|| HunspellHandle(Hunspell::new("dictionaries/en.aff", "dictionaries/en.dic")))
        .0
}

fn vi() -> &'static Hunspell {
    &VI.get_or_init(|| HunspellHandle(Hunspell::new("dictionaries/vi.aff", "dictionaries/vi.dic")))
        .0
}

fn checkers(lang: &str) -> Vec<&'static Hunspell> {
    match lang {
        "en" => vec![en()],
        "vi" => vec![vi()],
        _ => vec![en(), vi()],
    }
}

pub fn spellcheck(token: &str, whitelist: &HashSet<String>, lang: &str) -> (bool, Vec<String>) {
    if whitelist.contains(token) {
        return (false, Vec::new());
    }
    let hs = checkers(lang);
    if hs.iter().any(|h| h.check(token)) {
        return (false, Vec::new());
    }
    let mut suggestions = Vec::new();
    for h in hs {
        for s in h.suggest(token) {
            if !suggestions.contains(&s) {
                suggestions.push(s);
            }
        }
    }
    (true, suggestions)
}