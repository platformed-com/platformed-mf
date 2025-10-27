pub mod formatter;
pub mod parser;
pub mod types;

pub use formatter::{FormatError, format_message};
use icu::locale::Locale;
pub use parser::parse_message;
pub use types::{
    DateTimeExpression, DateTimeFormatType, DateTimeStyle, Message, MessageElement,
    NumberExpression, NumberFormatType, ParameterValue, Parameters, SelectCase, SelectExpression,
};

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
    locale: &Locale,
    message_str: &str,
    parameters: Parameters<'a>,
) -> Result<String, MessageFormatError> {
    let (_, message) = parse_message(message_str)?;
    let result = format_message(locale, &message, parameters)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use icu::locale::locale;

    use super::*;

    const EN_LOCALE: &Locale = &locale!("en");

    #[test]
    fn test_basic_interpolation() {
        let result = format(EN_LOCALE, "Hello {name}!", params!("name" => "World"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello World!");
    }

    #[test]
    fn test_multiple_parameters() {
        let result = format(
            EN_LOCALE,
            "{greeting} {name}{punctuation}",
            params!(
                "greeting" => "Hello",
                "name" => "Alice",
                "punctuation" => "!"
            ),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello Alice!");
    }

    #[test]
    fn test_no_parameters() {
        let result = format(EN_LOCALE, "Hello world!", params!());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello world!");
    }

    #[test]
    fn test_missing_parameter_error() {
        let result = format(EN_LOCALE, "Hello {name}!", params!());
        assert!(result.is_err());
    }

    #[test]
    fn test_tolgee_example_basic() {
        let result = format(
            EN_LOCALE,
            "You have {itemCount} items in your cart.",
            params!("itemCount" => "5"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 5 items in your cart.");
    }

    #[test]
    fn test_icu_example_basic() {
        let result = format(EN_LOCALE, "Hello {name}!", params!("name" => "John"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello John!");
    }

    #[test]
    fn test_plural_one_item() {
        let result = format(
            EN_LOCALE,
            "You have {count, plural, one{1 item} other{# items}} in your cart.",
            params!("count" => 1),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 1 item in your cart.");
    }

    #[test]
    fn test_plural_multiple_items() {
        let result = format(
            EN_LOCALE,
            "You have {count, plural, one{1 item} other{# items}} in your cart.",
            params!("count" => 5),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 5 items in your cart.");
    }

    #[test]
    fn test_plural_zero_items() {
        let result = format(
            EN_LOCALE,
            "{count, plural, zero{No items} one{1 item} other{# items}}",
            params!("count" => 0),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No items");
    }

    #[test]
    fn test_tolgee_plural_example() {
        let result = format(
            EN_LOCALE,
            "You have {itemCount, plural, one{# item} other{# items}} in your cart.",
            params!("itemCount" => 3),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "You have 3 items in your cart.");
    }

    #[test]
    fn test_icu_plural_example() {
        let result = format(
            EN_LOCALE,
            "{n, plural, one{# day} other{# days}}",
            params!("n" => 1),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1 day");
    }

    #[test]
    fn test_select_gender_male() {
        let result = format(
            EN_LOCALE,
            "{gender, select, male{He likes this.} female{She likes this.} other{They like this.}}",
            params!("gender" => "male"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "He likes this.");
    }

    #[test]
    fn test_select_gender_female() {
        let result = format(
            EN_LOCALE,
            "{gender, select, male{He likes this.} female{She likes this.} other{They like this.}}",
            params!("gender" => "female"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "She likes this.");
    }

    #[test]
    fn test_select_gender_fallback() {
        let result = format(
            EN_LOCALE,
            "{gender, select, male{He likes this.} female{She likes this.} other{They like this.}}",
            params!("gender" => "nonbinary"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "They like this.");
    }

    #[test]
    fn test_format_with_owned_strings() {
        let name = "Bob".to_string();
        let greeting = String::from("Hi");

        let result = format(
            EN_LOCALE,
            "{greeting}, {name}!",
            params!(
                "greeting" => greeting,
                "name" => name
            ),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hi, Bob!");
    }

    #[test]
    fn test_number_basic() {
        let result = format(EN_LOCALE, "{count, number}", params!("count" => 42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_number_integer() {
        let result = format(
            EN_LOCALE,
            "{count, number, integer}",
            params!("count" => "19.99"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "19");
    }

    #[test]
    fn test_number_percent() {
        let result = format(
            EN_LOCALE,
            "{ratio, number, percent}",
            params!("ratio" => "0.75"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "75%");
    }

    #[test]
    fn test_number_currency_usd() {
        let result = format(
            EN_LOCALE,
            "{price, number, currency}",
            params!("price" => "19.99"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "$19.99");
    }

    #[test]
    fn test_number_currency_eur() {
        let result = format(
            EN_LOCALE,
            "{price, number, currency/EUR}",
            params!("price" => 25),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "€25");
    }

    #[test]
    fn test_unparsed_input_rejected() {
        // Test that incomplete parameter syntax is rejected
        let result = format(EN_LOCALE, "Hello {name", params!("name" => "World"));
        assert!(result.is_err());

        // Test that invalid syntax after valid content is rejected
        let result2 = format(
            EN_LOCALE,
            "Valid {param} {unclosed",
            params!("param" => "value"),
        );
        assert!(result2.is_err());

        // Test that empty parameter name is rejected
        let result3 = format(EN_LOCALE, "Hello {}", params!());
        assert!(result3.is_err());
    }

    #[test]
    fn test_locale_specific_currency_formatting() {
        // Test that different locales produce different formatting
        let result_de = format(
            &locale!("de"),
            "{price, number, currency/EUR}",
            params!("price" => 1234),
        );
        let result_en = format(
            EN_LOCALE,
            "{price, number, currency/EUR}",
            params!("price" => 1234),
        );

        assert!(result_de.is_ok());
        assert!(result_en.is_ok());

        let formatted_de = result_de.unwrap();
        let formatted_en = result_en.unwrap();

        // German locale: symbol after amount with non-breaking space
        assert_eq!(formatted_de, "1.234\u{a0}€");
        // English locale: symbol before amount (no thousands separator for 4-digit numbers)
        assert_eq!(formatted_en, "€1,234");
    }

    // Exact matching tests (Tolgee documentation examples)
    #[test]
    fn test_plural_exact_zero() {
        let result = format(
            EN_LOCALE,
            "{dogsCount, plural, =0{No dogs} one{One dog is} other{# dogs are}} here.",
            params!("dogsCount" => 0),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No dogs here.");
    }

    #[test]
    fn test_plural_exact_vs_named_zero() {
        // Test that exact match takes precedence over named selector
        let result = format(
            EN_LOCALE,
            "{count, plural, =0{Exactly zero} zero{Named zero} other{Other}}",
            params!("count" => 0),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Exactly zero");
    }

    #[test]
    fn test_plural_exact_multiple_numbers() {
        let result = format(
            EN_LOCALE,
            "{n, plural, =0{no items} =1{one item} =2{a couple items} other{# items}}",
            params!("n" => 2),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "a couple items");
    }

    #[test]
    fn test_plural_exact_fallback_to_other() {
        let result = format(
            EN_LOCALE,
            "{n, plural, =0{no items} =1{one item} =2{a couple items} other{# items}}",
            params!("n" => 5),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5 items");
    }

    // All plural forms test (two, few, many)
    #[test]
    fn test_plural_form_two() {
        let result = format(
            EN_LOCALE,
            "{count, plural, one{# item} two{# pair} other{# items}}",
            params!("count" => 2),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2 pair");
    }

    #[test]
    fn test_plural_form_few() {
        // For English, 'few' form won't be selected by count, but we can test parsing
        let result = format(
            EN_LOCALE,
            "{count, plural, one{# item} few{# items (few)} other{# items}}",
            params!("count" => 3),
        );
        assert!(result.is_ok());
        // English doesn't use 'few', so should fall back to 'other'
        assert_eq!(result.unwrap(), "3 items");
    }

    #[test]
    fn test_plural_form_many() {
        // For English, 'many' form won't be selected by count, but we can test parsing
        let result = format(
            EN_LOCALE,
            "{count, plural, one{# item} many{# items (many)} other{# items}}",
            params!("count" => 10),
        );
        assert!(result.is_ok());
        // English doesn't use 'many', so should fall back to 'other'
        assert_eq!(result.unwrap(), "10 items");
    }

    #[test]
    fn test_plural_all_forms_parsing() {
        // Test that all plural forms can be parsed (even if not used for English)
        let result = format(
            EN_LOCALE,
            "{n, plural, =0{zero exact} zero{zero named} one{one} two{two} few{few} many{many} other{other}}",
            params!("n" => 1),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "one");
    }

    // Nested message tests (from Tolgee documentation)
    #[test]
    fn test_nested_select_with_plural() {
        let result = format(
            EN_LOCALE,
            "{gender, select, male{{count, plural, one{He has # item} other{He has # items}}} female{{count, plural, one{She has # item} other{She has # items}}} other{{count, plural, one{They have # item} other{They have # items}}}}",
            params!("gender" => "male", "count" => 1),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "He has 1 item");
    }

    #[test]
    fn test_nested_select_with_plural_multiple() {
        let result = format(
            EN_LOCALE,
            "{gender, select, male{{count, plural, one{He has # item} other{He has # items}}} female{{count, plural, one{She has # item} other{She has # items}}} other{{count, plural, one{They have # item} other{They have # items}}}}",
            params!("gender" => "female", "count" => 3),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "She has 3 items");
    }

    #[test]
    fn test_nested_plural_with_select() {
        let result = format(
            EN_LOCALE,
            "{count, plural, one{{gender, select, male{He saved} female{She saved} other{They saved}} one file} other{{gender, select, male{He saved} female{She saved} other{They saved}} # files}}",
            params!("count" => 1, "gender" => "female"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "She saved one file");
    }

    #[test]
    fn test_nested_plural_with_select_multiple() {
        let result = format(
            EN_LOCALE,
            "{count, plural, one{{gender, select, male{He saved} female{She saved} other{They saved}} one file} other{{gender, select, male{He saved} female{She saved} other{They saved}} # files}}",
            params!("count" => 5, "gender" => "male"),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "He saved 5 files");
    }

    #[test]
    fn test_tolgee_dogs_example_exact_zero() {
        // Direct example from Tolgee documentation
        let result = format(
            EN_LOCALE,
            "{dogsCount, plural, =0{No dogs} one{One dog is} other{# dogs are}} here.",
            params!("dogsCount" => 0),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "No dogs here.");
    }

    #[test]
    fn test_tolgee_dogs_example_one() {
        let result = format(
            EN_LOCALE,
            "{dogsCount, plural, =0{No dogs} one{One dog is} other{# dogs are}} here.",
            params!("dogsCount" => 1),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "One dog is here.");
    }

    #[test]
    fn test_tolgee_dogs_example_other() {
        let result = format(
            EN_LOCALE,
            "{dogsCount, plural, =0{No dogs} one{One dog is} other{# dogs are}} here.",
            params!("dogsCount" => 3),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "3 dogs are here.");
    }

    // Date/Time formatting tests
    #[test]
    fn test_date_short() {
        // Unix timestamp for Jan 1, 2024
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Date: {date, date, short}",
            params!("date" => timestamp),
        );
        assert!(result.is_ok());
        // The output will be locale-specific, so we just verify it doesn't error
        let formatted = result.unwrap();
        assert_eq!(formatted, "Date: 1/1/24");
    }

    #[test]
    fn test_date_medium() {
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Date: {date, date, medium}",
            params!("date" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "Date: Jan 1, 2024");
    }

    #[test]
    fn test_date_long() {
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Date: {date, date, long}",
            params!("date" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "Date: January 1, 2024");
    }

    #[test]
    fn test_date_full() {
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Date: {date, date, full}",
            params!("date" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "Date: Monday, January 1, 2024");
    }

    #[test]
    fn test_time_short() {
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Time: {time, time, short}",
            params!("time" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "Time: 12:00:00\u{202f}AM");
    }

    #[test]
    fn test_time_medium() {
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Time: {time, time, medium}",
            params!("time" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "Time: 12:00:00\u{202f}AM");
    }

    #[test]
    fn test_time_long() {
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Time: {time, time, long}",
            params!("time" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "Time: 12:00:00\u{202f}AM");
    }

    #[test]
    fn test_time_full() {
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Time: {time, time, full}",
            params!("time" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "Time: 12:00:00\u{202f}AM");
    }

    #[test]
    fn test_date_and_time_combined() {
        let timestamp = 1704067200i64; // 2024-01-01 00:00:00 UTC
        let result = format(
            EN_LOCALE,
            "Event on {date, date, medium} at {time, time, short}",
            params!("date" => timestamp, "time" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(!formatted.is_empty());
        assert!(formatted.starts_with("Event on "));
        assert!(formatted.contains(" at "));
    }

    #[test]
    fn test_datetime_with_text() {
        let timestamp = 1704067200i64;
        let result = format(
            EN_LOCALE,
            "The meeting is scheduled for {date, date, long}.",
            params!("date" => timestamp),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.starts_with("The meeting is scheduled for "));
        assert!(formatted.ends_with("."));
    }

    #[test]
    fn test_datetime_with_string_timestamp() {
        let result = format(
            EN_LOCALE,
            "Date: {date, date, short}",
            params!("date" => "1704067200"),
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(!formatted.is_empty());
        assert!(formatted.starts_with("Date: "));
    }

    #[test]
    fn test_datetime_locale_specific() {
        let timestamp = 1704067200i64;
        let result_en = format(
            EN_LOCALE,
            "{date, date, medium}",
            params!("date" => timestamp),
        );
        let result_de = format(
            &locale!("de"),
            "{date, date, medium}",
            params!("date" => timestamp),
        );

        assert!(result_en.is_ok());
        assert!(result_de.is_ok());

        // Different locales should produce different outputs
        // We don't assert exact strings as they depend on ICU4X data
        assert!(!result_en.unwrap().is_empty());
        assert!(!result_de.unwrap().is_empty());
    }
}
