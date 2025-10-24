#[derive(Debug, Clone, PartialEq)]
pub enum MessageElement {
    Text(String),
    Parameter(String),
    Plural(PluralExpression),
    Select(SelectExpression),
    Number(NumberExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluralExpression {
    pub parameter: String,
    pub cases: Vec<PluralCase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluralCase {
    pub selector: PluralSelector,
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectExpression {
    pub parameter: String,
    pub cases: Vec<SelectCase>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectCase {
    pub selector: String,
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberExpression {
    pub parameter: String,
    pub format_type: NumberFormatType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberFormatType {
    Number,        // Basic number formatting
    Integer,       // Integer formatting (no decimals)
    Percent,       // Percentage formatting
    Currency(String), // Currency formatting with optional currency code
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluralSelector {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
    Exact(i64),
}

impl PluralSelector {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "zero" => Some(PluralSelector::Zero),
            "one" => Some(PluralSelector::One),
            "two" => Some(PluralSelector::Two),
            "few" => Some(PluralSelector::Few),
            "many" => Some(PluralSelector::Many),
            "other" => Some(PluralSelector::Other),
            _ => {
                if let Ok(num) = s.parse::<i64>() {
                    Some(PluralSelector::Exact(num))
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub elements: Vec<MessageElement>,
}

impl Message {
    pub fn new(elements: Vec<MessageElement>) -> Self {
        Self { elements }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParameterValue<'a> {
    String(&'a str),
    Number(i64),
}

// Trait for types that can be used as parameter values without taking ownership
pub trait AsParameterValue {
    fn as_parameter_value<'a>(&'a self) -> ParameterValue<'a>;
}

impl AsParameterValue for &str {
    fn as_parameter_value<'a>(&'a self) -> ParameterValue<'a> {
        ParameterValue::String(self)
    }
}

impl AsParameterValue for i64 {
    fn as_parameter_value<'a>(&'a self) -> ParameterValue<'a> {
        ParameterValue::Number(*self)
    }
}

impl AsParameterValue for i32 {
    fn as_parameter_value<'a>(&'a self) -> ParameterValue<'a> {
        ParameterValue::Number(*self as i64)
    }
}

impl AsParameterValue for String {
    fn as_parameter_value<'a>(&'a self) -> ParameterValue<'a> {
        ParameterValue::String(self.as_str())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Parameters<'a> {
    pairs: &'a [(&'a str, ParameterValue<'a>)],
}

impl<'a> Parameters<'a> {
    pub fn empty() -> Self {
        Self { pairs: &[] }
    }

    pub fn from_slice(pairs: &'a [(&'a str, ParameterValue<'a>)]) -> Self {
        // Validate that all keys are distinct
        for (i, (key, _)) in pairs.iter().enumerate() {
            for (other_key, _) in pairs.iter().skip(i + 1) {
                if key == other_key {
                    panic!("Duplicate parameter key: {key}");
                }
            }
        }
        Self { pairs }
    }

    pub fn get(&self, key: &str) -> Option<&ParameterValue<'a>> {
        self.pairs.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }
}

// Convenience macro for creating parameters
#[macro_export]
macro_rules! params {
    () => {{
        $crate::types::Parameters::empty()
    }};
    ($($key:expr => $value:expr),+ $(,)?) => {
        $crate::types::Parameters::from_slice(&[
            $(($key, $crate::types::AsParameterValue::as_parameter_value(&$value)),)+
        ])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Duplicate parameter key: name")]
    fn test_duplicate_keys_panic() {
        Parameters::from_slice(&[
            ("name", ParameterValue::String("Alice")),
            ("age", ParameterValue::Number(25)),
            ("name", ParameterValue::String("Bob")),
        ]);
    }

    #[test]
    fn test_unique_keys_ok() {
        let params = Parameters::from_slice(&[
            ("name", ParameterValue::String("Alice")),
            ("age", ParameterValue::Number(25)),
            ("city", ParameterValue::String("NYC")),
        ]);

        assert_eq!(params.get("name"), Some(&ParameterValue::String("Alice")));
        assert_eq!(params.get("age"), Some(&ParameterValue::Number(25)));
        assert_eq!(params.get("city"), Some(&ParameterValue::String("NYC")));
        assert_eq!(params.get("unknown"), None);
    }

    #[test]
    fn test_params_macro_with_owned_string() {
        let name = "Alice".to_string();
        let city = String::from("New York");

        // Test by using params! directly in assertions
        let test_fn = |params: Parameters| {
            assert_eq!(params.get("name"), Some(&ParameterValue::String("Alice")));
            assert_eq!(params.get("age"), Some(&ParameterValue::Number(25)));
            assert_eq!(params.get("city"), Some(&ParameterValue::String("New York")));
        };

        test_fn(params!(
            "name" => name,
            "age" => 25,
            "city" => city
        ));
    }
}
