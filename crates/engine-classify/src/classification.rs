use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ClassScore {
    pub label: String,
    /// Points of evidence for the class.
    pub score: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Classification {
    /// The kind of document, or `unknown` when the evidence is too thin or ambiguous.
    pub label: String,
    /// From 0 to 1: how far the best class is ahead of the others, given how much evidence it has. 0 for `unknown`.
    pub confidence: f32,
    /// The words and phrases found that point to the chosen kind, so a person can check the reason.
    pub matched: Vec<String>,
    /// Every class that got any evidence, strongest first.
    pub scores: Vec<ClassScore>,
}
