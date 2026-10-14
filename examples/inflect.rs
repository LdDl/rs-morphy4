use rs_morphy4::MorphAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::args().nth(1).unwrap_or_else(|| "data/dicts_ru".into());
    let analyzer = MorphAnalyzer::load(directory)?;
    if let Some(parse) = analyzer.parse("кошка")?.first() {
        if let Some(form) = analyzer.inflect(parse, ["plur", "gent"])? {
            println!("{}", form.word);
        }
        for count in [1, 2, 5, 21] {
            if let Some(form) = analyzer.make_agree_with_number(parse, count)? {
                println!("{count} {}", form.word);
            }
        }
    }
    Ok(())
}
