pub fn parse_sort(
    sort: Option<String>,
    default_field: &'static str,
    allowed_fields: &[&'static str],
) -> (String, String) {
    let Some(sort) = sort else {
        return (default_field.to_string(), "desc".to_string());
    };
    let Some((field, dir)) = sort.split_once(':') else {
        return (default_field.to_string(), "desc".to_string());
    };
    let field = field.trim();
    let dir = dir.trim().to_ascii_lowercase();
    let dir = match dir.as_str() {
        "asc" => "asc",
        "desc" => "desc",
        _ => "desc",
    };
    if allowed_fields.contains(&field) {
        (field.to_string(), dir.to_string())
    } else {
        (default_field.to_string(), "desc".to_string())
    }
}
