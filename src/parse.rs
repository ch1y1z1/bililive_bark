use std::collections::HashMap;

pub fn parse(message: &str) -> Option<HashMap<String, String>> {
    Some(logfmt_parse::parse(message)?.into_iter().collect())
}
