use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub fn format_ts(ts: OffsetDateTime) -> String {
    ts.format(&Rfc3339)
        .unwrap_or_else(|_| ts.unix_timestamp().to_string())
}
