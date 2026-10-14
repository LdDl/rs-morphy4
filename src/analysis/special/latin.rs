const INFLECTION_ENDINGS: &[&str] = &["ов", "ам", "ами", "ах", "ями", "ях", "ей", "ой", "ую", "ом"];

fn has_latin(word: &str) -> bool {
    word.chars().any(|c| c.is_ascii_lowercase())
}

fn has_cyrillic(word: &str) -> bool {
    word.chars().any(|c| matches!(c, 'а'..='я' | 'ё'))
}

pub fn is_latin(word: &str) -> bool {
    let latin = has_latin(word);
    let cyrillic = has_cyrillic(word);
    if word.contains('-') && latin && cyrillic {
        !word.split('-').filter(|p| has_cyrillic(p)).any(|part| {
            INFLECTION_ENDINGS.iter().any(|ending| {
                part.strip_suffix(ending).is_some_and(|stem| stem.chars().count() > 2)
            })
        })
    } else {
        latin && !cyrillic
    }
}
