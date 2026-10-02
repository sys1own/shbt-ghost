//! ghost-audit — shared audit types for the 78-gate verification harness.

/// One audit gate result row for `verification_matrix.json`.
#[derive(Debug, Clone)]
pub struct Gate {
    pub id: &'static str,
    pub sector: &'static str,
    pub metric: &'static str,
    pub bound: &'static str,
    pub value: String,
    pub passed: bool,
}

impl Gate {
    pub fn new(
        id: &'static str,
        sector: &'static str,
        metric: &'static str,
        bound: &'static str,
        value: String,
        passed: bool,
    ) -> Self {
        Self { id, sector, metric, bound, value, passed }
    }

    /// Emit a compact JSON object (no serde dependency in the harness).
    pub fn to_json(&self) -> String {
        format!(
            "    {{\"id\": \"{}\", \"sector\": \"{}\", \"metric\": \"{}\", \"bound\": \"{}\", \"value\": \"{}\", \"status\": \"{}\"}}",
            self.id,
            self.sector,
            self.metric,
            self.bound,
            self.value.replace('\\', "\\\\").replace('"', "\\\""),
            if self.passed { "PASS" } else { "FAIL" },
        )
    }
}
