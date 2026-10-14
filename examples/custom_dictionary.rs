use rs_morphy4::{DictionaryData, MorphAnalyzer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dictionary = DictionaryData {
        metadata: include_bytes!("../data/dicts_ru/meta.json"),
        grammemes: include_bytes!("../data/dicts_ru/grammemes.json"),
        tags: include_bytes!("../data/dicts_ru/gramtab-opencorpora-int.json"),
        suffixes: include_bytes!("../data/dicts_ru/suffixes.json"),
        paradigms: include_bytes!("../data/dicts_ru/paradigms.array"),
        words: include_bytes!("../data/dicts_ru/words.dawg"),
    };
    let analyzer = MorphAnalyzer::new(dictionary)?;
    for parse in analyzer.parse("кошки")? {
        println!("{}: {}", parse.normal_form, parse.tag);
    }
    Ok(())
}
