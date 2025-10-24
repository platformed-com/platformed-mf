use crate::types::{
    Message, MessageElement, ParameterValue, Parameters, PluralExpression, PluralSelector, SelectExpression, NumberFormatType,
};
use icu::decimal::FixedDecimalFormatter;
use icu::decimal::options::FixedDecimalFormatterOptions;
use icu::experimental::dimension::currency::formatter::{CurrencyFormatter, CurrencyCode};
use icu::locid::Locale;
use writeable::Writeable;

#[derive(Debug, Clone, PartialEq)]
pub enum FormatError {
    MissingParameter(String),
    InvalidParameterType(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::MissingParameter(param) => {
                write!(f, "Missing parameter: {param}")
            }
            FormatError::InvalidParameterType(param) => {
                write!(f, "Invalid parameter type for: {param}")
            }
        }
    }
}

impl std::error::Error for FormatError {}

fn select_plural_case(plural_expr: &PluralExpression, count: i64) -> Option<&Message> {
    // First, look for exact number matches
    for case in &plural_expr.cases {
        if let PluralSelector::Exact(n) = case.selector {
            if n == count {
                return Some(&case.message);
            }
        }
    }

    // Then apply basic English plural rules
    let rule = match count {
        0 => PluralSelector::Zero,
        1 => PluralSelector::One,
        2 => PluralSelector::Two,
        _ => PluralSelector::Other,
    };

    // Look for the matching rule
    for case in &plural_expr.cases {
        if case.selector == rule {
            return Some(&case.message);
        }
    }

    // Fall back to "other" if available
    for case in &plural_expr.cases {
        if case.selector == PluralSelector::Other {
            return Some(&case.message);
        }
    }

    None
}

fn substitute_hash_placeholder(text: &str, count: i64) -> String {
    text.replace('#', &count.to_string())
}

fn select_case<'a>(select_expr: &'a SelectExpression, value: &str) -> Option<&'a Message> {
    // First, look for exact matches
    for case in &select_expr.cases {
        if case.selector == value {
            return Some(&case.message);
        }
    }

    // Fall back to "other" if available
    for case in &select_expr.cases {
        if case.selector == "other" {
            return Some(&case.message);
        }
    }

    None
}

fn format_number(value: f64, format_type: &NumberFormatType, locale: &Locale) -> Result<String, FormatError> {
    use fixed_decimal::FixedDecimal;

    match format_type {
        NumberFormatType::Number => {
            let formatter = FixedDecimalFormatter::try_new(&locale, FixedDecimalFormatterOptions::default())
                .map_err(|_| FormatError::InvalidParameterType("number".to_string()))?;

            let fixed_decimal = if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
                FixedDecimal::from(value as i64)
            } else {
                let value_str = value.to_string();
                value_str.parse::<FixedDecimal>()
                    .map_err(|_| FormatError::InvalidParameterType("number".to_string()))?
            };

            Ok(formatter.format(&fixed_decimal).to_string())
        }
        NumberFormatType::Integer => {
            let formatter = FixedDecimalFormatter::try_new(&locale, FixedDecimalFormatterOptions::default())
                .map_err(|_| FormatError::InvalidParameterType("number".to_string()))?;

            let fixed_decimal = FixedDecimal::from(value as i64);
            Ok(formatter.format(&fixed_decimal).to_string())
        }
        NumberFormatType::Percent => {
            // For now, use simple formatting until we add proper percent formatter
            let percentage = (value * 100.0) as i64;
            Ok(format!("{}%", percentage))
        }
        NumberFormatType::Currency(currency) => {
            let currency_formatter = CurrencyFormatter::try_new(&locale, Default::default())
                .map_err(|_| FormatError::InvalidParameterType("currency".to_string()))?;

            let fixed_decimal = if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
                FixedDecimal::from(value as i64)
            } else {
                let value_str = value.to_string();
                value_str.parse::<FixedDecimal>()
                    .map_err(|_| FormatError::InvalidParameterType("currency".to_string()))?
            };

            // Create currency code dynamically from any valid 3-character currency code
            let currency_code = if currency.len() == 3 && currency.chars().all(|c| c.is_ascii_alphabetic()) {
                let currency_upper = currency.to_uppercase();
                // Parse the currency string into a TinyAsciiStr and wrap in CurrencyCode
                match currency_upper.parse() {
                    Ok(tiny_str) => CurrencyCode(tiny_str),
                    Err(_) => return Err(FormatError::InvalidParameterType(format!("Invalid currency code: {}", currency))),
                }
            } else {
                return Err(FormatError::InvalidParameterType(format!("Currency code must be 3 ASCII letters: {}", currency)));
            };

            let formatted = currency_formatter.format_fixed_decimal(&fixed_decimal, currency_code);

            // Use write_to method to convert FormattedCurrency to String
            let mut result = String::new();
            formatted.write_to(&mut result)
                .map_err(|_| FormatError::InvalidParameterType("currency formatting".to_string()))?;
            Ok(result)
        }
    }
}

pub fn format_message<'a>(
    message: &Message,
    parameters: Parameters<'a>,
    locale: &Locale,
) -> Result<String, FormatError> {
    let mut result = String::new();

    for element in &message.elements {
        match element {
            MessageElement::Text(text) => {
                result.push_str(text);
            }
            MessageElement::Parameter(param_name) => match parameters.get(param_name) {
                Some(ParameterValue::String(value)) => result.push_str(value),
                Some(ParameterValue::Number(value)) => result.push_str(&value.to_string()),
                None => return Err(FormatError::MissingParameter(param_name.clone())),
            },
            MessageElement::Plural(plural_expr) => {
                let count = match parameters.get(&plural_expr.parameter) {
                    Some(ParameterValue::Number(n)) => *n,
                    Some(ParameterValue::String(s)) => match s.parse::<i64>() {
                        Ok(n) => n,
                        Err(_) => {
                            return Err(FormatError::InvalidParameterType(
                                plural_expr.parameter.clone(),
                            ));
                        }
                    },
                    None => {
                        return Err(FormatError::MissingParameter(plural_expr.parameter.clone()));
                    }
                };

                if let Some(selected_message) = select_plural_case(plural_expr, count) {
                    let formatted_submessage = format_message(selected_message, parameters, locale)?;
                    let with_substitutions =
                        substitute_hash_placeholder(&formatted_submessage, count);
                    result.push_str(&with_substitutions);
                }
            }
            MessageElement::Select(select_expr) => {
                let value = match parameters.get(&select_expr.parameter) {
                    Some(ParameterValue::String(s)) => *s,
                    Some(ParameterValue::Number(_)) => return Err(FormatError::InvalidParameterType(select_expr.parameter.clone())),
                    None => return Err(FormatError::MissingParameter(select_expr.parameter.clone())),
                };

                if let Some(selected_message) = select_case(select_expr, value) {
                    let formatted_submessage = format_message(selected_message, parameters, locale)?;
                    result.push_str(&formatted_submessage);
                }
            }
            MessageElement::Number(number_expr) => {
                let number_value = match parameters.get(&number_expr.parameter) {
                    Some(ParameterValue::Number(n)) => *n as f64,
                    Some(ParameterValue::String(s)) => {
                        match s.parse::<f64>() {
                            Ok(n) => n,
                            Err(_) => return Err(FormatError::InvalidParameterType(number_expr.parameter.clone())),
                        }
                    }
                    None => return Err(FormatError::MissingParameter(number_expr.parameter.clone())),
                };

                let formatted_number = format_number(number_value, &number_expr.format_type, locale)?;
                result.push_str(&formatted_number);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params;
    use crate::types::{MessageElement, PluralCase, PluralExpression, PluralSelector, SelectCase, SelectExpression, NumberExpression, NumberFormatType};

    #[test]
    fn test_format_text_only() {
        let message = Message::new(vec![MessageElement::Text("Hello world".to_string())]);
        let params = params!();

        let result = format_message(&message, params);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello world");
    }

    #[test]
    fn test_format_single_parameter() {
        let message = Message::new(vec![
            MessageElement::Text("Hello ".to_string()),
            MessageElement::Parameter("name".to_string()),
        ]);

        let result = format_message(&message, params!("name" => "Alice"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello Alice");
    }

    #[test]
    fn test_format_multiple_parameters() {
        let message = Message::new(vec![
            MessageElement::Text("Hello ".to_string()),
            MessageElement::Parameter("firstName".to_string()),
            MessageElement::Text(" ".to_string()),
            MessageElement::Parameter("lastName".to_string()),
            MessageElement::Text("!".to_string()),
        ]);
        let result = format_message(&message, params!(
            "firstName" => "Alice",
            "lastName" => "Johnson"
        ));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello Alice Johnson!");
    }

    #[test]
    fn test_format_missing_parameter() {
        let message = Message::new(vec![
            MessageElement::Text("Hello ".to_string()),
            MessageElement::Parameter("name".to_string()),
        ]);
        let params = params!();

        let result = format_message(&message, params);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            FormatError::MissingParameter("name".to_string())
        );
    }

    #[test]
    fn test_format_plural_one() {
        let plural_expr = PluralExpression {
            parameter: "count".to_string(),
            cases: vec![
                PluralCase {
                    selector: PluralSelector::One,
                    message: Message::new(vec![MessageElement::Text("1 item".to_string())]),
                },
                PluralCase {
                    selector: PluralSelector::Other,
                    message: Message::new(vec![MessageElement::Text("# items".to_string())]),
                },
            ],
        };
        let message = Message::new(vec![MessageElement::Plural(plural_expr)]);
        let result = format_message(&message, params!("count" => 1));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1 item");
    }

    #[test]
    fn test_format_plural_other() {
        let plural_expr = PluralExpression {
            parameter: "count".to_string(),
            cases: vec![
                PluralCase {
                    selector: PluralSelector::One,
                    message: Message::new(vec![MessageElement::Text("1 item".to_string())]),
                },
                PluralCase {
                    selector: PluralSelector::Other,
                    message: Message::new(vec![MessageElement::Text("# items".to_string())]),
                },
            ],
        };
        let message = Message::new(vec![MessageElement::Plural(plural_expr)]);
        let result = format_message(&message, params!("count" => 5));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5 items");
    }

    #[test]
    fn test_format_plural_with_context() {
        let plural_expr = PluralExpression {
            parameter: "count".to_string(),
            cases: vec![
                PluralCase {
                    selector: PluralSelector::One,
                    message: Message::new(vec![MessageElement::Text("1 item".to_string())]),
                },
                PluralCase {
                    selector: PluralSelector::Other,
                    message: Message::new(vec![MessageElement::Text("# items".to_string())]),
                },
            ],
        };
        let message = Message::new(vec![
            MessageElement::Text("You have ".to_string()),
            MessageElement::Plural(plural_expr),
            MessageElement::Text(" in your cart.".to_string()),
        ]);
        let result = format_message(&message, params!("count" => 3));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 3 items in your cart.");
    }

    #[test]
    fn test_format_select_male() {
        let select_expr = SelectExpression {
            parameter: "gender".to_string(),
            cases: vec![
                SelectCase {
                    selector: "male".to_string(),
                    message: Message::new(vec![MessageElement::Text("He likes this.".to_string())]),
                },
                SelectCase {
                    selector: "female".to_string(),
                    message: Message::new(vec![MessageElement::Text("She likes this.".to_string())]),
                },
                SelectCase {
                    selector: "other".to_string(),
                    message: Message::new(vec![MessageElement::Text("They like this.".to_string())]),
                },
            ],
        };
        let message = Message::new(vec![MessageElement::Select(select_expr)]);

        let result = format_message(&message, params!("gender" => "male"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "He likes this.");
    }

    #[test]
    fn test_format_select_female() {
        let select_expr = SelectExpression {
            parameter: "gender".to_string(),
            cases: vec![
                SelectCase {
                    selector: "male".to_string(),
                    message: Message::new(vec![MessageElement::Text("He likes this.".to_string())]),
                },
                SelectCase {
                    selector: "female".to_string(),
                    message: Message::new(vec![MessageElement::Text("She likes this.".to_string())]),
                },
                SelectCase {
                    selector: "other".to_string(),
                    message: Message::new(vec![MessageElement::Text("They like this.".to_string())]),
                },
            ],
        };
        let message = Message::new(vec![MessageElement::Select(select_expr)]);

        let result = format_message(&message, params!("gender" => "female"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "She likes this.");
    }

    #[test]
    fn test_format_select_fallback_to_other() {
        let select_expr = SelectExpression {
            parameter: "gender".to_string(),
            cases: vec![
                SelectCase {
                    selector: "male".to_string(),
                    message: Message::new(vec![MessageElement::Text("He likes this.".to_string())]),
                },
                SelectCase {
                    selector: "female".to_string(),
                    message: Message::new(vec![MessageElement::Text("She likes this.".to_string())]),
                },
                SelectCase {
                    selector: "other".to_string(),
                    message: Message::new(vec![MessageElement::Text("They like this.".to_string())]),
                },
            ],
        };
        let message = Message::new(vec![MessageElement::Select(select_expr)]);

        let result = format_message(&message, params!("gender" => "nonbinary"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "They like this.");
    }

    #[test]
    fn test_format_number_basic() {
        let number_expr = NumberExpression {
            parameter: "count".to_string(),
            format_type: NumberFormatType::Number,
        };
        let message = Message::new(vec![MessageElement::Number(number_expr)]);

        let result = format_message(&message, params!("count" => 42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_format_number_decimal() {
        let number_expr = NumberExpression {
            parameter: "price".to_string(),
            format_type: NumberFormatType::Number,
        };
        let message = Message::new(vec![MessageElement::Number(number_expr)]);

        let result = format_message(&message, params!("price" => "19.99"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "19.99");
    }

    #[test]
    fn test_format_number_integer() {
        let number_expr = NumberExpression {
            parameter: "count".to_string(),
            format_type: NumberFormatType::Integer,
        };
        let message = Message::new(vec![MessageElement::Number(number_expr)]);

        let result = format_message(&message, params!("count" => "19.99"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "19");
    }

    #[test]
    fn test_format_number_percent() {
        let number_expr = NumberExpression {
            parameter: "ratio".to_string(),
            format_type: NumberFormatType::Percent,
        };
        let message = Message::new(vec![MessageElement::Number(number_expr)]);

        let result = format_message(&message, params!("ratio" => "0.75"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "75%");
    }

    #[test]
    fn test_format_number_currency_usd() {
        let number_expr = NumberExpression {
            parameter: "price".to_string(),
            format_type: NumberFormatType::Currency("USD".to_string()),
        };
        let message = Message::new(vec![MessageElement::Number(number_expr)]);

        let result = format_message(&message, params!("price" => "19.99"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "$19.99");
    }

    #[test]
    fn test_format_number_currency_eur() {
        let number_expr = NumberExpression {
            parameter: "price".to_string(),
            format_type: NumberFormatType::Currency("EUR".to_string()),
        };
        let message = Message::new(vec![MessageElement::Number(number_expr)]);

        let result = format_message(&message, params!("price" => 25));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "€25");
    }

    #[test]
    fn test_format_number_currency_any_valid_code() {
        let number_expr = NumberExpression {
            parameter: "price".to_string(),
            format_type: NumberFormatType::Currency("SEK".to_string()),
        };
        let message = Message::new(vec![MessageElement::Number(number_expr)]);

        let result = format_message(&message, params!("price" => 100));
        assert!(result.is_ok());
        // ICU4X should handle SEK (Swedish Krona) even though we didn't hardcode it
        let formatted = result.unwrap();
        assert!(formatted.contains("100") || formatted.contains("SEK"));
    }
}
