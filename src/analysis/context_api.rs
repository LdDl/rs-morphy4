use crate::analysis::analyzer::MorphAnalyzer;
use crate::analysis::context::decision::decide;
use crate::analysis::context::rules::evaluate;
use crate::analysis::context::types::ContextAnalysis;
use crate::dictionary::DictionaryError;

impl MorphAnalyzer {
    /// Analyzes caller-supplied tokens using bounded local grammatical rules.
    ///
    /// Punctuation must be separate tokens and sentence boundaries must be preserved.
    /// Each token keeps its complete parse output. Ties between lemma/POS pairs
    /// leave selected empty; no baseline fallback is applied. A selected pair can
    /// still be wrong or contain several grammatical analyses. This is not a sentence parser.
    /// Only word analyses are cached; contextual decisions are computed per call.
    ///
    /// # Errors
    /// Returns the underlying dictionary error if any token cannot be analyzed.
    ///
    /// ```
    /// use rs_morphy4::MorphAnalyzer;
    /// let morph = MorphAnalyzer::default();
    /// let result = morph.parse_context(["Мы", "будем", "вести", "переговоры", "."])?;
    /// let reading = result[2].decision.selected.as_ref().unwrap();
    /// assert_eq!((reading.lemma.as_str(), reading.pos.as_str()), ("вести", "INFN"));
    /// assert!(morph.parse_context(["Я", "плачу", "."])?[1].decision.selected.is_none());
    /// # Ok::<(), rs_morphy4::dictionary::DictionaryError>(())
    /// ```
    pub fn parse_context<I, S>(&self, tokens: I) -> Result<Vec<ContextAnalysis>, DictionaryError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let tokens: Vec<String> = tokens
            .into_iter()
            .map(|token| token.as_ref().to_owned())
            .collect();
        let candidates = tokens
            .iter()
            .map(|token| self.parse(token))
            .collect::<Result<Vec<_>, _>>()?;
        let normalized: Vec<String> = tokens
            .iter()
            .map(|token| token.trim().to_lowercase())
            .collect();
        let decisions: Vec<_> = (0..tokens.len())
            .map(|target| {
                decide(
                    &candidates[target],
                    evaluate(&normalized, &candidates, target),
                )
            })
            .collect();
        Ok(tokens
            .into_iter()
            .zip(candidates)
            .zip(decisions)
            .map(|((token, candidates), decision)| ContextAnalysis {
                token,
                candidates,
                decision,
            })
            .collect())
    }
}
