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
