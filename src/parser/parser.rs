use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0, multispace1},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};
use std::str::FromStr;

use crate::ast::AST;
use crate::error::ParseError;

// Parsing a string literal.
pub fn string_literal(input: &str) -> IResult<&str, AST> {
    let parse_str = delimited(tag("\""), take_while(|c| c != '"'), tag("\""));
    map(parse_str, |s: &str| AST::String(s.to_string()))(input)
}

pub fn name(input: &str) -> IResult<&str, AST> {
    let parse_str = delimited(tag("\""), take_while(|c| c != ' '), tag("\""));
    map(parse_str, |s: &str| AST::String(s.to_string()))(input)
}

// Parsing an identifier.
pub fn identifier(input: &str) -> IResult<&str, AST> {
    map(
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        |id: &str| AST::Identifier(id.to_string()),
    )(input)
}

// Parsing an integer literal.
pub fn integer(input: &str) -> IResult<&str, AST> {
    map(map_res(digit1, |s: &str| i32::from_str(s)), AST::Integer)(input)
}

// Parsing a float literal.
pub fn float(input: &str) -> IResult<&str, AST> {
    let float_parser = recognize(tuple((digit1, tag("."), digit1)));
    map(
        map_res(float_parser, |s: &str| f64::from_str(s)),
        AST::Float,
    )(input)
}

// Parsing a boolean literal.
pub fn boolean(input: &str) -> IResult<&str, AST> {
    alt((
        map(tag_no_case("true"), |_| AST::Bool(true)),
        map(tag_no_case("false"), |_| AST::Bool(false)),
    ))(input)
}

// Parsing a parenthesized expression.
pub fn parenthesized_expression(input: &str) -> IResult<&str, AST> {
    delimited(tag("("), math_expression, tag(")"))(input)
}

// Parsing an array literal.
pub fn array_literal(input: &str) -> IResult<&str, AST> {
    let (input, _) = tag("[")(input)?;
    let (input, elements) = separated_list0(
        preceded(multispace0, tag(",")),
        preceded(
            multispace0,
            alt((math_expression, string_literal, dictionary_literal)),
        ),
    )(input)?;
    let (input, _) = preceded(multispace0, tag("]"))(input)?;
    Ok((input, AST::Array(elements)))
}
// Parsing a dictionary literal.
pub fn dictionary_literal(input: &str) -> IResult<&str, AST> {
    let (input, _) = tag("{")(input)?;
    let (input, pairs) = separated_list0(
        preceded(multispace0, tag(",")),
        preceded(
            multispace0,
            separated_pair(
                preceded(multispace0, alt((math_expression, string_literal))),
                preceded(multispace0, tag(":")),
                preceded(multispace0, alt((math_expression, string_literal))),
            ),
        ),
    )(input)?;
    let (input, _) = preceded(multispace0, tag("}"))(input)?;
    Ok((input, AST::Dictionary(pairs)))
}

// Parsing a factor (a basic unit in an expression).
pub fn factor(input: &str) -> IResult<&str, AST> {
    alt((
        float,
        integer,
        boolean,
        identifier,
        string_literal,
        array_literal,
        dictionary_literal,
        parenthesized_expression,
    ))(input)
}

// Parsing a term (a factor possibly followed by * or / operations).
pub fn term(input: &str) -> IResult<&str, AST> {
    let (input, init) = factor(input)?;
    let (input, res) = many0(tuple((
        preceded(multispace0, alt((tag("*"), tag("/")))),
        preceded(multispace0, factor),
    )))(input)?;

    let acc = res.into_iter().fold(init, |acc, (op, val)| AST::BinaryOp {
        left: Box::new(acc),
        op: op.to_string(),
        right: Box::new(val),
    });
    Ok((input, acc))
}

// Parsing a math expression (a term possibly followed by + or - operations).
pub fn math_expression(input: &str) -> IResult<&str, AST> {
    let (input, init) = term(input)?;
    let (input, res) = many0(tuple((
        preceded(multispace0, alt((tag("+"), tag("-")))),
        preceded(multispace0, term),
    )))(input)?;

    let acc = res.into_iter().fold(init, |acc, (op, val)| AST::BinaryOp {
        left: Box::new(acc),
        op: op.to_string(),
        right: Box::new(val),
    });
    Ok((input, acc))
}

// Parsing a return statement.
pub fn return_stmt(input: &str) -> IResult<&str, AST> {
    let (input, _) = tag("return")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, expr) = math_expression(input)?;
    Ok((input, AST::Return(Box::new(expr))))
}

// Parsing a write statement.
pub fn write_stmt(input: &str) -> IResult<&str, AST> {
    let (input, _) = tag("write")(input)?;
    let (input, _) = multispace1(input)?;
    // Парсим список выражений, разделённых запятыми
    let (input, expr_list) = separated_list0(
        preceded(multispace0, tag(",")),
        preceded(multispace0, alt((
            function_call,  // Вызов функции
            math_expression,
            string_literal,
            identifier,  // Поддержка идентификаторов (например, переменных)
        )))
    )(input)?;
    Ok((input, AST::Write(expr_list)))
}




// Parsing a comparison operator.
pub fn comparison_operator(input: &str) -> IResult<&str, &str> {
    alt((
        tag("="),
        tag("!="),
        tag("<="),
        tag(">="),
        tag("<"),
        tag(">"),
    ))(input)
}

// Parsing a comparison expression.
pub fn comparison_expression(input: &str) -> IResult<&str, AST> {
    let (input, left) = math_expression(input)?;
    let (input, res) = many0(tuple((
        preceded(multispace0, comparison_operator),
        preceded(multispace0, math_expression),
    )))(input)?;

    let acc = res.into_iter().fold(left, |acc, (op, val)| AST::BinaryOp {
        left: Box::new(acc),
        op: op.to_string(),
        right: Box::new(val),
    });
    Ok((input, acc))
}

fn parse_arguments(input: &str) -> IResult<&str, Vec<AST>> {
    let (input, args) = delimited(
        char('('),
        separated_list0(
            preceded(multispace0, char(',')),
            preceded(multispace0, factor),  // Заменяем identifier на factor, чтобы поддерживались все типы данных
        ),
        char(')'),
    )(input)?;

    Ok((input, args))
}

pub fn function(input: &str) -> IResult<&str, AST> {
    // Parse the name and arguments
    let (input, (name, args)) = tuple((
        take_while1(|c: char| c.is_alphanumeric() || c == '_'),
        preceded(multispace0, parse_arguments),
    ))(input)?;

    // Ignore any whitespace between the name with arguments and the opening brace
    let (input, _) = multispace0(input)?;
    let (input, _) = char('{')(input)?;

    // Parse the contents of the block
    let (input, elements) = many0(preceded(multispace0, statement))(input)?;

    // Ignore any whitespace between the block contents and the closing brace
    let (input, _) = delimited(multispace0, char('}'), multispace0)(input)?;

    // Construct the AST with the function name, arguments, and body
    Ok((
        input,
        AST::Function {
            name: name.to_string(),
            args: Box::new(AST::FunctionArgs(args)), // Use Box<AST> here
            body: Box::new(AST::Block(elements)),
        },
    ))
}

pub fn function_call(input: &str) -> IResult<&str, AST> {
    let (input, name) = identifier(input)?;
    let (input, args) = parse_arguments(input)?;
    Ok((
        input,
        AST::FunctionCall {
            name: match name {
                AST::Identifier(id) => id,
                _ => unreachable!(),
            },
            args,
        },
    ))
}

// Parsing a coincide statement.
pub fn coincide(input: &str) -> IResult<&str, AST> {
    let (input, _) = tag("coincide")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, expr) = identifier(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace1(input)?;

    let (input, cases) = many0(tuple((
        preceded(multispace0, math_expression),
        preceded(multispace1, tag("then")),
        preceded(multispace1, statement),
    )))(input)?;

    let (input, default) = opt(preceded(
        multispace0,
        tuple((tag("default"), preceded(multispace1, statement))),
    ))(input)?;

    let cases = cases
        .into_iter()
        .map(|(condition, _, action)| (condition, action))
        .collect();

    let default = default.map(|(_, action)| Box::new(action));

    Ok((
        input,
        AST::Coincide {
            expr: Box::new(expr),
            cases,
            default,
        },
    ))
}

// Parsing an if-else statement.
pub fn if_else(input: &str) -> IResult<&str, AST> {
    // Parse the "if" keyword and the condition
    let (input, _) = tag("if")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, condition) = comparison_expression(input)?;

    // Ignore whitespace and expect the opening brace for the block
    let (input, _) = multispace0(input)?;
    let (input, _) = char('{')(input)?;

    // Parse the statements inside the block
    let (input, then_branch) = many0(preceded(multispace0, statement))(input)?;

    // Expect the closing brace for the block
    let (input, _) = char('}')(input)?;

    // Parse optional "else" or "elif" branches
    let mut elif_branches = vec![];
    let mut input = input;

    loop {
        let (i, _) = multispace0(input)?;
        let (i, next_token) = opt(alt((tag("else"), tag("elif"))))(i)?;

        match next_token {
            Some("elif") => {
                // Parse "elif" condition and block
                let (i, _) = multispace1(i)?;
                let (i, elif_condition) = comparison_expression(i)?;
                let (i, _) = multispace0(i)?;
                let (i, _) = char('{')(i)?;
                let (i, elif_branch) = many0(preceded(multispace0, statement))(i)?;
                let (i, _) = char('}')(i)?;
                elif_branches.push((elif_condition, AST::Block(elif_branch)));
                input = i;
            }
            Some("else") => {
                // Parse "else" block
                let (i, _) = multispace0(i)?;
                let (i, _) = char('{')(i)?;
                let (i, else_branch) = many0(preceded(multispace0, statement))(i)?;
                let (i, _) = char('}')(i)?;
                return Ok((
                    i,
                    AST::IfElse {
                        condition: Box::new(condition),
                        then_branch: Box::new(AST::Block(then_branch)),
                        elif_branches,
                        else_branch: Some(Box::new(AST::Block(else_branch))),
                    },
                ));
            }
            _ => break,
        }
    }

    Ok((
        input,
        AST::IfElse {
            condition: Box::new(condition),
            then_branch: Box::new(AST::Block(then_branch)),
            elif_branches,
            else_branch: None,
        },
    ))
}


// Parsing a variable assignment.
pub fn variable_assign(input: &str) -> IResult<&str, AST> {
    let (input, name) = identifier(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("is")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, value) = alt((
        math_expression,
        string_literal,
        array_literal,
        dictionary_literal,
    ))(input)?;
    Ok((
        input,
        AST::VariableAssign {
            name: match name {
                AST::Identifier(id) => id,
                _ => unreachable!(),
            },
            value: Box::new(value),
        },
    ))
}

// Parsing a statement (includes all possible statements).
pub fn statement(input: &str) -> IResult<&str, AST> {
    preceded(
        multispace0,
        alt((
            return_stmt,
            write_stmt,
            variable_assign,
            function,
            function_call, // Added function call parsing
            if_else,
            coincide,
        )),
    )(input)
}

// Parsing a program (a series of statements).
pub fn program(input: &str) -> IResult<&str, Vec<AST>> {
    many0(preceded(multispace0, statement))(input)
}

// Parsing the program and returning the result or a parse error.
pub fn parse_program(input: &str) -> Result<AST, ParseError> {
    match program(input) {
        Ok((remaining, ast)) => {
            if !remaining.trim().is_empty() {
                let line = input.lines().take_while(|l| !remaining.contains(l)).count() + 1;
                return Err(ParseError::UnknownToken {
                    token: remaining.trim().to_string(),
                    line,
                });
            }
            Ok(AST::Program(ast))
        }
        Err(nom::Err::Error(_err)) => {
            let line = input.lines().count();
            Err(ParseError::SyntaxError {
                message: format!("Failed to parse program at line {}", line),
                line,
            })
        }
        Err(err) => Err(ParseError::SyntaxError {
            message: format!("Failed to parse program: {:?}", err),
            line: input.lines().count(),
        }),
    }
}

// Helper function for case-insensitive tag matching.
fn tag_no_case(tag: &'static str) -> impl Fn(&str) -> IResult<&str, &str> {
    move |input: &str| {
        let tag_lower = tag.to_lowercase();
        let input_lower = input.to_lowercase();

        if input_lower.starts_with(&tag_lower) {
            Ok((&input[tag.len()..], &input[..tag.len()]))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        }
    }
}
