use regex::Regex;
use std::sync::LazyLock;

/// Shared snake_case validation pattern for entity names (datasets, assets).
pub static SNAKE_CASE_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").unwrap());
