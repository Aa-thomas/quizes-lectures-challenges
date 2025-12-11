use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum MyError {
    NotFound { key: String },
}

pub fn get_owned(map: &HashMap<String, String>, key: &str) -> Result<String, MyError> {
    map.get(key).cloned().ok_or_else(|| MyError::NotFound {
        key: key.to_string(),
    })
}

#[cfg(test)]
mod mc1_tests {
    use super::*;

    #[test]
    fn returns_ok_value() {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("name".to_string(), "aaron".to_string());

        let result = get_owned(&map, "name");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "aaron");
    }

    #[test]
    fn returns_my_error() {
        let map: HashMap<String, String> = HashMap::new();

        let result = get_owned(&map, "nonexistent");

        assert!(result.is_err());

        match result {
            Err(MyError::NotFound { key }) => {
                assert_eq!(key, "nonexistent");
            }
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }
}
