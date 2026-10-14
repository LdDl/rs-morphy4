use rs_morphy4::MorphAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let directory = args.next().unwrap_or_else(|| "data/dicts_ru".into());
    let word = args.next().unwrap_or_else(|| "стали".into());
    let analyzer = MorphAnalyzer::load(directory)?;
    for parse in analyzer.parse(&word)? {
        println!(
            "{}\t{}\t{}\t{}",
            parse.word, parse.normal_form, parse.tag, parse.score
        );
    }
    Ok(())
}
