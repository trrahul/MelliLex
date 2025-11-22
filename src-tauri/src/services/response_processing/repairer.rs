pub struct JsonRepairer;

impl JsonRepairer {
    pub fn repair_truncated_json(input: &str) -> Option<RepairResult> {
        let mut stack: Vec<char> = Vec::new();
        let mut repaired = String::with_capacity(input.len() + 8);
        repaired.push_str(input);

        let mut in_string = false;
        let mut escape = false;

        for ch in input.chars() {
            if escape {
                escape = false;
                continue;
            }

            if in_string {
                match ch {
                    '\\' => escape = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }

            match ch {
                '"' => in_string = true,
                '{' => stack.push('}'),
                '[' => stack.push(']'),
                '}' | ']' => {
                    if stack.pop() != Some(ch) {
                        // Mismatched brackets - can't repair
                        return None;
                    }
                }
                _ => {}
            }
        }

        // If stack is empty and not in a string, JSON is already balanced
        if stack.is_empty() && !in_string {
            return None;
        }

        let mut appended = 0;

        // Close any unclosed string first
        if in_string {
            repaired.push('"');
            appended += 1;
        }

        // Append missing closers
        appended += stack.len();
        while let Some(closer) = stack.pop() {
            repaired.push(closer);
        }

        Some(RepairResult {
            text: repaired,
            appended,
        })
    }
}

pub struct RepairResult {
    pub text: String,
    pub appended: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_truncated_object() {
        let input = r#"{"key": "value""#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_some());
        let repair = result.unwrap();
        assert_eq!(repair.text, r#"{"key": "value"}"#);
        assert_eq!(repair.appended, 1);
    }

    #[test]
    fn repair_truncated_array() {
        let input = r#"["item1", "item2""#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_some());
        let repair = result.unwrap();
        assert_eq!(repair.text, r#"["item1", "item2"]"#);
        assert_eq!(repair.appended, 1);
    }

    #[test]
    fn repair_nested_truncation() {
        let input = r#"{"outer": {"inner": "value""#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_some());
        let repair = result.unwrap();
        assert_eq!(repair.text, r#"{"outer": {"inner": "value"}}"#);
        assert_eq!(repair.appended, 2);
    }

    #[test]
    fn repair_already_valid() {
        let input = r#"{"key": "value"}"#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_none(), "Valid JSON should not be repaired");
    }

    #[test]
    fn repair_with_string_escapes() {
        let input = r#"{"key": "val\"ue with escaped quote""#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_some());
        let repair = result.unwrap();
        assert_eq!(repair.text, r#"{"key": "val\"ue with escaped quote"}"#);
        assert_eq!(repair.appended, 1);
    }

    #[test]
    fn repair_mismatched_brackets_returns_none() {
        let input = r#"{"key": ]"#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_none(), "Mismatched brackets can't be repaired");
    }

    #[test]
    fn repair_complex_nested() {
        let input = r#"{
  "domain_explorations": [
    {
      "domain": "Academic"
    }
  "#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_some());
        let repair = result.unwrap();
        assert_eq!(repair.appended, 2); // Missing ] and }
        // Verify it's valid JSON by checking balance
        assert!(repair.text.ends_with("]}"));
    }

    #[test]
    fn repair_empty_string() {
        let result = JsonRepairer::repair_truncated_json("");
        assert!(result.is_none());
    }

    #[test]
    fn repair_truncated_mid_string() {
        // Simulates AI response cut off mid-string value
        let input = r#"{"patterns": [{"description": "This is a long descrip"#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_some());
        let repair = result.unwrap();
        // Should close the string, then the object, then the array, then the outer object
        assert_eq!(repair.text, r#"{"patterns": [{"description": "This is a long descrip"}]}"#);
        assert_eq!(repair.appended, 4); // " + } + ] + }
    }

    #[test]
    fn repair_truncated_mid_unicode_string() {
        // Simulates truncation in non-ASCII content (like Hindi text from the error)
        let input = r#"{"desc": "किसी कथन को वापस"#;
        let result = JsonRepairer::repair_truncated_json(input);
        assert!(result.is_some());
        let repair = result.unwrap();
        assert_eq!(repair.text, r#"{"desc": "किसी कथन को वापस"}"#);
        assert_eq!(repair.appended, 2); // " + }
    }
}
