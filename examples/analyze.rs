use rs_morphy4::MorphAnalyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let word = std::env::args().nth(1).unwrap_or_else(|| "стали".into());
    let analyzer = MorphAnalyzer::default();
    for parse in analyzer.parse(&word)? {
        println!(
            "{}\t{}\t{}\t{}",
            parse.word, parse.normal_form, parse.tag, parse.score
        );
    }
    Ok(())
}
