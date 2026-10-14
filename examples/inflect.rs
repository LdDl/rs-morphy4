use rs_morphy4::MorphAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let analyzer = MorphAnalyzer::default();
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
