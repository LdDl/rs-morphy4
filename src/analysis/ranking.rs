use crate::analysis::parse::Parse;

pub fn rank(parses: &mut [Parse]) {
    parses.sort_by_key(|p| {
        let pos = match p.tag.pos.as_str() {
            "NOUN" => 0,
            "VERB" | "INFN" => 1,
            "ADJF" | "NUMR" | "NPRO" => 2,
            "ADJS" | "ADVB" | "PRTF" => 3,
            "PRTS" | "GRND" | "PRED" => 4,
            _ => 10,
        };
        (
            p.score < 1.0,
            pos,
            !p.tag.contains("nomn"),
            p.tag.grammemes.len(),
        )
    });
}

/// Orders by word/tag frequency, retaining the heuristic order for equal frequencies.
pub fn statistical(
    word: &str,
    parses: Vec<Parse>,
    model: &crate::dictionary::WordTagProbabilities,
) -> Result<Vec<Parse>, crate::dictionary::DictionaryError> {
    let mut weighted = parses
        .into_iter()
        .map(|parse| {
            model
                .frequency(word, &parse.tag)
                .map(|frequency| (frequency, parse))
        })
        .collect::<Result<Vec<_>, _>>()?;
    weighted.sort_by_key(|(frequency, _)| std::cmp::Reverse(*frequency));
    Ok(weighted.into_iter().map(|(_, parse)| parse).collect())
}
