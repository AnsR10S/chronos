use crate::shell::state::variables;

pub fn expand_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| expand_string(arg))
        .filter(|arg| !arg.is_empty())
        .collect()
}

fn expand_string(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(&'{') = chars.peek() {
                chars.next();

                let mut var_name = String::new();

                while let Some(&next_c) = chars.peek() {
                    if next_c == '}' {
                        chars.next();
                        break;
                    } else {
                        var_name.push(chars.next().unwrap());
                    }
                }

                if !var_name.is_empty() {
                    if let Some(val) = variables::get_variable(&var_name) {
                        result.push_str(&val);
                    }
                }
            } else {
                let mut var_name = String::new();

                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_alphanumeric() || next_c == '_' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if !var_name.is_empty() {
                    if let Some(val) = variables::get_variable(&var_name) {
                        result.push_str(&val);
                    }
                } else {
                    result.push('$');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::state::variables;

    #[test]
    fn test_expand_no_variables() {
        assert_eq!(expand_string("hello world"), "hello world");
    }

    #[test]
    fn test_expand_simple_var() {
        variables::set_variable("EXP_T2_FOO".to_string(), "bar".to_string());
        assert_eq!(expand_string("$EXP_T2_FOO"), "bar");
    }

    #[test]
    fn test_expand_braced_var() {
        variables::set_variable("EXP_T3_FOO".to_string(), "bar".to_string());
        assert_eq!(expand_string("${EXP_T3_FOO}"), "bar");
    }

    #[test]
    fn test_expand_unknown_var() {
        assert_eq!(expand_string("$UNKNOWN_VAR_XYZ"), "");
    }

    #[test]
    fn test_expand_unknown_braced_var() {
        assert_eq!(expand_string("${UNKNOWN_VAR_XYZ}"), "");
    }

    #[test]
    fn test_expand_var_in_middle() {
        variables::set_variable("EXP_T6_X".to_string(), "mid".to_string());
        assert_eq!(expand_string("pre${EXP_T6_X}suf"), "premidsuf");
    }

    #[test]
    fn test_expand_multiple_vars() {
        variables::set_variable("EXP_T7_A".to_string(), "1".to_string());
        variables::set_variable("EXP_T7_B".to_string(), "2".to_string());
        assert_eq!(expand_string("$EXP_T7_A and $EXP_T7_B"), "1 and 2");
    }

    #[test]
    fn test_expand_adjacent_vars() {
        variables::set_variable("EXP_T8_X".to_string(), "a".to_string());
        variables::set_variable("EXP_T8_Y".to_string(), "b".to_string());
        assert_eq!(expand_string("$EXP_T8_X$EXP_T8_Y"), "ab");
    }

    #[test]
    fn test_expand_bare_dollar() {
        assert_eq!(expand_string("cost is $"), "cost is $");
    }

    #[test]
    fn test_expand_dollar_followed_by_space() {
        assert_eq!(expand_string("$ hello"), "$ hello");
    }

    #[test]
    fn test_expand_underscore_in_name() {
        variables::set_variable("EXP_T11_MY_VAR".to_string(), "val".to_string());
        assert_eq!(expand_string("$EXP_T11_MY_VAR"), "val");
    }

    #[test]
    fn test_expand_numeric_in_name() {
        variables::set_variable("EXP_T12_VAR1".to_string(), "one".to_string());
        assert_eq!(expand_string("$EXP_T12_VAR1"), "one");
    }

    #[test]
    fn test_expand_args_filters_empty() {
        let args = vec!["$UNKNOWN_VAR_13".to_string(), "good".to_string()];
        let expanded = expand_args(&args);
        assert_eq!(expanded, vec!["good".to_string()]);
    }

    #[test]
    fn test_expand_args_preserves_nonempty() {
        variables::set_variable("EXP_T14_V".to_string(), "value".to_string());
        let args = vec!["$EXP_T14_V".to_string()];
        let expanded = expand_args(&args);
        assert_eq!(expanded, vec!["value".to_string()]);
    }

    #[test]
    fn test_expand_no_recursive() {
        variables::set_variable("EXP_T15_A".to_string(), "$EXP_T15_B".to_string());
        variables::set_variable("EXP_T15_B".to_string(), "deep".to_string());
        assert_eq!(expand_string("$EXP_T15_A"), "$EXP_T15_B");
    }

    #[test]
    fn test_expand_unterminated_brace() {
        assert_eq!(expand_string("${UNCLOSED"), "");
    }

    #[test]
    fn test_expand_empty_braces() {
        assert_eq!(expand_string("${}"), "");
    }

    #[test]
    fn test_expand_mixed_braced_unbraced() {
        variables::set_variable("EXP_T18_X".to_string(), "1".to_string());
        variables::set_variable("EXP_T18_Y".to_string(), "2".to_string());
        assert_eq!(expand_string("${EXP_T18_X}.$EXP_T18_Y"), "1.2");
    }

    #[test]
    fn test_expand_value_with_special_chars() {
        variables::set_variable("EXP_T19_V".to_string(), "a b c".to_string());
        assert_eq!(expand_string("$EXP_T19_V"), "a b c");
    }

    #[test]
    fn test_expand_value_with_dollar() {
        variables::set_variable("EXP_T20_V".to_string(), "$100".to_string());
        assert_eq!(expand_string("$EXP_T20_V"), "$100");
    }

    #[test]
    fn test_expansion_depth_single_level() {
        variables::set_variable("EXP_T21_X".to_string(), "single".to_string());
        assert_eq!(expand_string("$EXP_T21_X"), "single");
    }

    #[test]
    fn test_expansion_depth_no_nesting() {
        variables::set_variable("EXP_T22_X".to_string(), "$EXP_T22_Y".to_string());
        variables::set_variable("EXP_T22_Y".to_string(), "nested".to_string());
        assert_eq!(expand_string("$EXP_T22_X"), "$EXP_T22_Y");
    }

    #[test]
    fn test_expansion_stress_many_vars() {
        let mut input = String::new();
        let mut expected = String::new();
        for i in 0..100 {
            let var_name = format!("EXP_T23_VAR{}", i);
            let val = format!("val{}", i);
            variables::set_variable(var_name.clone(), val.clone());
            input.push_str(&format!("${} ", var_name));
            expected.push_str(&format!("{} ", val));
        }
        assert_eq!(expand_string(&input), expected);
    }
}
