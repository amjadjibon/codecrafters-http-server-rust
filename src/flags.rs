use std::collections::HashMap;

pub fn parse_flags(args: &[String]) -> HashMap<String, String> {
    let mut flags = HashMap::new();
    let mut i = 0;
    while i < args.len() {
        if args[i].starts_with("--") {
            let key = args[i].trim_start_matches("--").to_string();
            let value = args[i + 1].to_string();
            flags.insert(key, value);
        }
        i += 1;
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flags() {
        let args = vec![
            "program_name".to_string(),
            "--key1".to_string(),
            "value1".to_string(),
            "--key2".to_string(),
            "value2".to_string(),
        ];
        let expected = vec![
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(parse_flags(&args), expected);
    }
}
