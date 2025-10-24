use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0},
    combinator::map,
    multi::{many0, many1},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::types::{Message, MessageElement, PluralExpression, PluralCase, PluralSelector, SelectExpression, SelectCase, NumberExpression, NumberFormatType};

fn parameter_name(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn simple_parameter(input: &str) -> IResult<&str, MessageElement> {
    map(
        delimited(
            char('{'),
            delimited(multispace0, parameter_name, multispace0),
            char('}'),
        ),
        |name| MessageElement::Parameter(name.to_string()),
    )(input)
}

fn plural_selector(input: &str) -> IResult<&str, PluralSelector> {
    map(
        take_while1(|c: char| c.is_alphanumeric()),
        |s: &str| PluralSelector::parse(s).unwrap_or(PluralSelector::Other),
    )(input)
}


fn case_content(input: &str) -> IResult<&str, Message> {
    delimited(
        char('{'),
        map(many0(alt((number_expression, select_expression, plural_expression, simple_parameter, text_segment_in_case))), Message::new),
        char('}'),
    )(input)
}

fn text_segment_in_case(input: &str) -> IResult<&str, MessageElement> {
    map(
        take_while1(|c: char| c != '{' && c != '}'),
        |text: &str| MessageElement::Text(text.to_string()),
    )(input)
}

fn plural_case(input: &str) -> IResult<&str, PluralCase> {
    map(
        tuple((
            delimited(multispace0, plural_selector, multispace0),
            case_content,
        )),
        |(selector, message)| PluralCase { selector, message },
    )(input)
}

fn select_case(input: &str) -> IResult<&str, SelectCase> {
    map(
        tuple((
            delimited(multispace0, select_selector, multispace0),
            case_content,
        )),
        |(selector, message)| SelectCase { selector, message },
    )(input)
}

fn select_selector(input: &str) -> IResult<&str, String> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |s: &str| s.to_string(),
    )(input)
}

fn plural_expression(input: &str) -> IResult<&str, MessageElement> {
    map(
        delimited(
            char('{'),
            tuple((
                delimited(multispace0, parameter_name, multispace0),
                preceded(
                    tuple((char(','), multispace0, tag("plural"), multispace0, char(','))),
                    delimited(multispace0, many1(plural_case), multispace0),
                ),
            )),
            char('}'),
        ),
        |(param, cases)| {
            MessageElement::Plural(PluralExpression {
                parameter: param.to_string(),
                cases,
            })
        },
    )(input)
}

fn select_expression(input: &str) -> IResult<&str, MessageElement> {
    map(
        delimited(
            char('{'),
            tuple((
                delimited(multispace0, parameter_name, multispace0),
                preceded(
                    tuple((char(','), multispace0, tag("select"), multispace0, char(','))),
                    delimited(multispace0, many1(select_case), multispace0),
                ),
            )),
            char('}'),
        ),
        |(param, cases)| {
            MessageElement::Select(SelectExpression {
                parameter: param.to_string(),
                cases,
            })
        },
    )(input)
}

fn number_expression(input: &str) -> IResult<&str, MessageElement> {
    map(
        delimited(
            char('{'),
            tuple((
                delimited(multispace0, parameter_name, multispace0),
                preceded(
                    tuple((char(','), multispace0, tag("number"))),
                    alt((
                        preceded(
                            tuple((multispace0, char(','), multispace0)),
                            number_format_type,
                        ),
                        map(multispace0, |_| NumberFormatType::Number),
                    )),
                ),
            )),
            char('}'),
        ),
        |(param, format_type)| {
            MessageElement::Number(NumberExpression {
                parameter: param.to_string(),
                format_type,
            })
        },
    )(input)
}

fn number_format_type(input: &str) -> IResult<&str, NumberFormatType> {
    alt((
        map(tag("integer"), |_| NumberFormatType::Integer),
        map(tag("percent"), |_| NumberFormatType::Percent),
        map(
            preceded(tag("currency"),
                alt((
                    preceded(char('/'), map(take_while1(|c: char| c.is_alphanumeric()), |s: &str| s.to_string())),
                    map(tag(""), |_| "USD".to_string()),
                ))
            ),
            NumberFormatType::Currency,
        ),
        map(tag(""), |_| NumberFormatType::Number),
    ))(input)
}

fn text_segment(input: &str) -> IResult<&str, MessageElement> {
    map(
        take_while1(|c: char| c != '{'),
        |text: &str| MessageElement::Text(text.to_string()),
    )(input)
}

fn message_element(input: &str) -> IResult<&str, MessageElement> {
    alt((number_expression, select_expression, plural_expression, simple_parameter, text_segment))(input)
}

pub fn parse_message(input: &str) -> IResult<&str, Message> {
    map(many0(message_element), |elements| {
        Message::new(elements)
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let result = parse_message("Hello world");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 1);
        assert_eq!(message.elements[0], MessageElement::Text("Hello world".to_string()));
    }

    #[test]
    fn test_parse_simple_parameter() {
        let result = parse_message("{name}");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 1);
        assert_eq!(message.elements[0], MessageElement::Parameter("name".to_string()));
    }

    #[test]
    fn test_parse_mixed_content() {
        let result = parse_message("Hello {name}!");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 3);
        assert_eq!(message.elements[0], MessageElement::Text("Hello ".to_string()));
        assert_eq!(message.elements[1], MessageElement::Parameter("name".to_string()));
        assert_eq!(message.elements[2], MessageElement::Text("!".to_string()));
    }

    #[test]
    fn test_parse_multiple_parameters() {
        let result = parse_message("Hello {firstName} {lastName}!");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 5);
        assert_eq!(message.elements[0], MessageElement::Text("Hello ".to_string()));
        assert_eq!(message.elements[1], MessageElement::Parameter("firstName".to_string()));
        assert_eq!(message.elements[2], MessageElement::Text(" ".to_string()));
        assert_eq!(message.elements[3], MessageElement::Parameter("lastName".to_string()));
        assert_eq!(message.elements[4], MessageElement::Text("!".to_string()));
    }

    #[test]
    fn test_parse_simple_plural() {
        let result = parse_message("{count, plural, one{1 item} other{# items}}");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 1);

        if let MessageElement::Plural(plural_expr) = &message.elements[0] {
            assert_eq!(plural_expr.parameter, "count");
            assert_eq!(plural_expr.cases.len(), 2);

            assert_eq!(plural_expr.cases[0].selector, PluralSelector::One);
            assert_eq!(plural_expr.cases[0].message.elements.len(), 1);
            assert_eq!(plural_expr.cases[0].message.elements[0], MessageElement::Text("1 item".to_string()));

            assert_eq!(plural_expr.cases[1].selector, PluralSelector::Other);
            assert_eq!(plural_expr.cases[1].message.elements.len(), 1);
            assert_eq!(plural_expr.cases[1].message.elements[0], MessageElement::Text("# items".to_string()));
        } else {
            panic!("Expected plural expression");
        }
    }

    #[test]
    fn test_parse_plural_with_text() {
        let result = parse_message("You have {count, plural, one{1 item} other{# items}} in your cart.");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 3);

        assert_eq!(message.elements[0], MessageElement::Text("You have ".to_string()));
        assert!(matches!(message.elements[1], MessageElement::Plural(_)));
        assert_eq!(message.elements[2], MessageElement::Text(" in your cart.".to_string()));
    }

    #[test]
    fn test_parse_simple_select() {
        let result = parse_message("{gender, select, male{He likes this.} female{She likes this.} other{They like this.}}");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 1);

        if let MessageElement::Select(select_expr) = &message.elements[0] {
            assert_eq!(select_expr.parameter, "gender");
            assert_eq!(select_expr.cases.len(), 3);

            assert_eq!(select_expr.cases[0].selector, "male");
            assert_eq!(select_expr.cases[0].message.elements.len(), 1);
            assert_eq!(select_expr.cases[0].message.elements[0], MessageElement::Text("He likes this.".to_string()));

            assert_eq!(select_expr.cases[1].selector, "female");
            assert_eq!(select_expr.cases[1].message.elements[0], MessageElement::Text("She likes this.".to_string()));

            assert_eq!(select_expr.cases[2].selector, "other");
            assert_eq!(select_expr.cases[2].message.elements[0], MessageElement::Text("They like this.".to_string()));
        } else {
            panic!("Expected select expression");
        }
    }

    #[test]
    fn test_parse_number_basic() {
        let result = parse_message("{count, number}");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 1);

        if let MessageElement::Number(number_expr) = &message.elements[0] {
            assert_eq!(number_expr.parameter, "count");
            assert_eq!(number_expr.format_type, NumberFormatType::Number);
        } else {
            panic!("Expected number expression");
        }
    }

    #[test]
    fn test_parse_number_integer() {
        let result = parse_message("{count, number, integer}");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 1);

        if let MessageElement::Number(number_expr) = &message.elements[0] {
            assert_eq!(number_expr.parameter, "count");
            assert_eq!(number_expr.format_type, NumberFormatType::Integer);
        } else {
            panic!("Expected number expression");
        }
    }

    #[test]
    fn test_parse_number_currency_eur() {
        let result = parse_message("{price, number, currency/EUR}");
        assert!(result.is_ok());
        let (_, message) = result.unwrap();
        assert_eq!(message.elements.len(), 1);

        if let MessageElement::Number(number_expr) = &message.elements[0] {
            assert_eq!(number_expr.parameter, "price");
            assert_eq!(number_expr.format_type, NumberFormatType::Currency("EUR".to_string()));
        } else {
            panic!("Expected number expression");
        }
    }
}