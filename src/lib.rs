pub mod formatter;
pub mod parser;
pub mod types;

pub use formatter::{FormatError, format_message};
pub use parser::parse_message;
pub use types::{Message, MessageElement, ParameterValue, Parameters, SelectExpression, SelectCase, NumberExpression, NumberFormatType};
pub use icu::locid::Locale;

#[derive(Debug)]
pub enum MessageFormatError {
    ParseError(String),
    FormatError(FormatError),
}

impl std::fmt::Display for MessageFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageFormatError::ParseError(msg) => write!(f, "Parse error: {msg}"),
            MessageFormatError::FormatError(err) => write!(f, "Format error: {err}"),
        }
    }
}

impl std::error::Error for MessageFormatError {}

impl From<nom::Err<nom::error::Error<&str>>> for MessageFormatError {
    fn from(err: nom::Err<nom::error::Error<&str>>) -> Self {
        MessageFormatError::ParseError(format!("{err:?}"))
    }
}

impl From<FormatError> for MessageFormatError {
    fn from(err: FormatError) -> Self {
        MessageFormatError::FormatError(err)
    }
}

pub fn format<'a>(
    message_str: &str,
    parameters: Parameters<'a>,
) -> Result<String, MessageFormatError> {
    let (_, message) = parse_message(message_str)?;
    let result = format_message(&message, parameters)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_interpolation() {
        let result = format("Hello {name}!", params!("name" => "World"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World!");
    }

    #[test]
    fn test_multiple_parameters() {
        let result = format("{greeting} {name}{punctuation}", params!(
            "greeting" => "Hello",
            "name" => "Alice",
            "punctuation" => "!"
        ));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello Alice!");
    }

    #[test]
    fn test_no_parameters() {
        let result = format("Hello world!", params!());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello world!");
    }

    #[test]
    fn test_missing_parameter_error() {
        let result = format("Hello {name}!", params!());
        assert!(result.is_err());
    }

    #[test]
    fn test_tolgee_example_basic() {
        let result = format("You have {itemCount} items in your cart.", params!("itemCount" => "5"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 5 items in your cart.");
    }

    #[test]
    fn test_icu_example_basic() {
        let result = format("Hello {name}!", params!("name" => "John"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello John!");
    }

    #[test]
    fn test_plural_one_item() {
        let result = format(
            "You have {count, plural, one{1 item} other{# items}} in your cart.",
            params!("count" => 1),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 1 item in your cart.");
    }

    #[test]
    fn test_plural_multiple_items() {
        let result = format(
            "You have {count, plural, one{1 item} other{# items}} in your cart.",
            params!("count" => 5),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 5 items in your cart.");
    }

    #[test]
    fn test_plural_zero_items() {
        let result = format(
            "{count, plural, zero{No items} one{1 item} other{# items}}",
            params!("count" => 0),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No items");
    }

    #[test]
    fn test_tolgee_plural_example() {
        let result = format(
            "You have {itemCount, plural, one{# item} other{# items}} in your cart.",
            params!("itemCount" => 3),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 3 items in your cart.");
    }

    #[test]
    fn test_icu_plural_example() {
        let result = format("{n, plural, one{# day} other{# days}}", params!("n" => 1));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1 day");
    }

    #[test]
    fn test_select_gender_male() {
        let result = format("{gender, select, male{He likes this.} female{She likes this.} other{They like this.}}", params!("gender" => "male"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "He likes this.");
    }

    #[test]
    fn test_select_gender_female() {
        let result = format("{gender, select, male{He likes this.} female{She likes this.} other{They like this.}}", params!("gender" => "female"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "She likes this.");
    }

    #[test]
    fn test_select_gender_fallback() {
        let result = format("{gender, select, male{He likes this.} female{She likes this.} other{They like this.}}", params!("gender" => "nonbinary"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "They like this.");
    }

    #[test]
    fn test_format_with_owned_strings() {
        let name = "Bob".to_string();
        let greeting = String::from("Hi");

        let result = format("{greeting}, {name}!", params!(
            "greeting" => greeting,
            "name" => name
        ));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hi, Bob!");
    }

    #[test]
    fn test_number_basic() {
        let result = format("{count, number}", params!("count" => 42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_number_integer() {
        let result = format("{count, number, integer}", params!("count" => "19.99"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "19");
    }

    #[test]
    fn test_number_percent() {
        let result = format("{ratio, number, percent}", params!("ratio" => "0.75"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "75%");
    }

    #[test]
    fn test_number_currency_usd() {
        let result = format("{price, number, currency}", params!("price" => "19.99"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "$19.99");
    }

    #[test]
    fn test_number_currency_eur() {
        let result = format("{price, number, currency/EUR}", params!("price" => 25));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "€25");
    }
}
