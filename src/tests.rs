#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn test_math_expression() {
        let input = "a + b - c * (2.5 + 2)";
        let (_, ast) = parser::math_expression(input).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn test_return_stmt() {
        let input = "return a + b - c * (2 + 2.5)\n";
        let (_, ast) = parser::return_stmt(input).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn test_write_stmt_with_string() {
        let input = "write \"hello\"";
        let (_, ast) = parser::write_stmt(input).unwrap();
        assert_eq!(ast, AST::Write(Box::new(AST::String("hello".to_string()))));
    }

    #[test]
    fn test_write_stmt_with_expression() {
        let input = "write a + b";
        let (_, ast) = parser::write_stmt(input).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn test_float_parsing() {
        let input = "3.14";
        let (_, ast) = parser::float(input).unwrap();
        assert_eq!(ast, AST::Float(3.14));
    }

    #[test]
    fn test_boolean_parsing() {
        let input = "true";
        let (_, ast) = parser::boolean(input).unwrap();
        assert_eq!(ast, AST::Bool(true));

        let input = "false";
        let (_, ast) = parser::boolean(input).unwrap();
        assert_eq!(ast, AST::Bool(false));
    }

    #[test]
    fn test_case_insensitive_boolean_parsing() {
        let input = "True";
        let (_, ast) = parser::boolean(input).unwrap();
        assert_eq!(ast, AST::Bool(true));

        let input = "False";
        let (_, ast) = parser::boolean(input).unwrap();
        assert_eq!(ast, AST::Bool(false));
    }

    #[test]
    fn test_array_of_objects_parsing() {
        let input = "[{a: 1, b: 2}, {a: 1, b: 2}, {a: 1, b: 2}]";
        let (_, ast) = parser::array_literal(input).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn test_write_array_of_objects() {
        let input = "write [{a: 1, b: 2}, {a: 1, b: 2}, {a: 1, b: 2}]";
        let (_, ast) = parser::write_stmt(input).unwrap();
        println!("{:#?}", ast);
    }
}
