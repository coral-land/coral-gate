use std::collections::BTreeMap;

/// TODO: Move to shared or utils
pub fn generate_lables() -> Option<BTreeMap<std::string::String, std::string::String>> {
    let mut labels: BTreeMap<String, String> = BTreeMap::new();

    labels.insert("created-by".into(), "coralgate".into());

    Some(labels)
}
