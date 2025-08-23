use std::collections::HashMap;

use logfmt::Pair;

pub fn parse(message: &str) -> HashMap<String, String> {
    let pairs = logfmt::parse(message);
    let mut map = HashMap::new();
    for Pair { key, val } in pairs {
        map.insert(key, val.unwrap_or_default());
    }
    map
}
