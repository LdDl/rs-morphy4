pub const METADATA: &[u8] = include_bytes!("../../data/dicts_ru/meta.json");
pub const GRAMMEMES: &[u8] = include_bytes!("../../data/dicts_ru/grammemes.json");
pub const TAGS: &[u8] = include_bytes!("../../data/dicts_ru/gramtab-opencorpora-int.json");
pub const SUFFIXES: &[u8] = include_bytes!("../../data/dicts_ru/suffixes.json");
pub const PARADIGMS: &[u8] = include_bytes!("../../data/dicts_ru/paradigms.array");
pub const WORDS: &[u8] = include_bytes!("../../data/dicts_ru/words.dawg");
pub const PREDICTION_SUFFIXES: &[&[u8]] = &[
    include_bytes!("../../data/dicts_ru/prediction-suffixes-0.dawg"),
    include_bytes!("../../data/dicts_ru/prediction-suffixes-1.dawg"),
    include_bytes!("../../data/dicts_ru/prediction-suffixes-2.dawg"),
];
