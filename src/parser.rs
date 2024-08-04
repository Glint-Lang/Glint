use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{digit1, multispace0, multispace1},
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

// Parsing an identifier.
pub fn identifier(input: &str) -> IResult<&str, AST> {
    map(take_while1(|c: char| c.is_alphanumeric() || c == '_'), |id: &str| {
        AST::Identifier(id.to_string())
    })(input)
}

// Parsing an integer literal.
pub fn integer(input: &str) -> IResult<&str, AST> {
    map(map_res(digit1, |s: &str| i32::from_str(s)), AST::Integer)(input)
}

// Parsing a float literal.
pub fn float(input: &str) -> IResult<&str, AST> {
    let float_parser = recognize(tuple((digit1, tag("."), digit1)));
    map(map_res(float_parser, |s: &str| f64::from_str(s)), AST::Float)(input)
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
        preceded(multispace0, alt((
            math_expression,
            string_literal,
            dictionary_literal
        )))
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
                preceded(multispace0, alt((
                    math_expression,
                    string_literal
                ))),
                preceded(multispace0, tag(":")),
                preceded(multispace0, alt((
                    math_expression,
                    string_literal
                )))
            )
        )
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
    let (input, res) = many0(tuple((preceded(multispace0, alt((tag("*"), tag("/")))), preceded(multispace0, factor))))(input)?;

    let acc = res.into_iter().fold(init, |acc, (op, val)| {
        AST::BinaryOp {
            left: Box::new(acc),
            op: op.to_string(),
            right: Box::new(val),
        }
    });
    Ok((input, acc))
}

// Parsing a math expression (a term possibly followed by + or - operations).
pub fn math_expression(input: &str) -> IResult<&str, AST> {
    let (input, init) = term(input)?;
    let (input, res) = many0(tuple((
        preceded(multispace0, alt((tag("+"), tag("-")))),
        preceded(multispace0, term)
    )))(input)?;

    let acc = res.into_iter().fold(init, |acc, (op, val)| {
        AST::BinaryOp {
            left: Box::new(acc),
            op: op.to_string(),
            right: Box::new(val),
        }
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
    let (input, expr) = alt((math_expression, string_literal, array_literal, dictionary_literal))(input)?;
    Ok((input, AST::Write(Box::new(expr))))
}

// Parsing a comparison operator.
pub fn comparison_operator(input: &str) -> IResult<&str, &str> {
    alt((tag("="), tag("!="), tag("<="), tag(">="), tag("<"), tag(">")))(input)
}

// Parsing a comparison expression.
pub fn comparison_expression(input: &str) -> IResult<&str, AST> {
    let (input, left) = math_expression(input)?;
    let (input, res) = many0(tuple((preceded(multispace0, comparison_operator), preceded(multispace0, math_expression))))(input)?;

    let acc = res.into_iter().fold(left, |acc, (op, val)| {
        AST::BinaryOp {
            left: Box::new(acc),
            op: op.to_string(),
            right: Box::new(val),
        }
    });
    Ok((input, acc))
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

    Ok((input, AST::Coincide {
        expr: Box::new(expr),
        cases,
        default,
    }))
}

// Parsing an if-else statement.
pub fn if_else(input: &str) -> IResult<&str, AST> {
    let (input, _) = tag("if")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, condition) = comparison_expression(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, then_branch) = statement(input)?;

    let mut elif_branches = vec![];
    let mut input = input;

    loop {
        let (i, _) = multispace0(input)?;
        let (i, elif) = alt((tag("elif"), tag("else")))(i)?;

        if elif == "elif" {
            let (i, _) = multispace1(i)?;
            let (i, elif_condition) = comparison_expression(i)?;
            let (i, _) = tag(":")(i)?;
            let (i, _) = multispace1(i)?;
            let (i, elif_branch) = statement(i)?;
            elif_branches.push((elif_condition, elif_branch));
            input = i;
        } else if elif == "else" {
            let (i, _) = tag(":")(i)?;
            let (i, _) = multispace1(i)?;
            let (i, else_branch) = statement(i)?;
            return Ok((i, AST::IfElse {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                elif_branches,
                else_branch: Some(Box::new(else_branch)),
            }));
        } else {
            break;
        }
    }

    Ok((input, AST::IfElse {
        condition: Box::new(condition),
        then_branch: Box::new(then_branch),
        elif_branches,
        else_branch: None,
    }))
}

// Parsing a function definition.
pub fn function(input: &str) -> IResult<&str, AST> {
    let (input, _) = tag("function")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, name) = identifier(input)?;
    let (input, args) = delimited(
        tag("("),
        separated_list0(
            preceded(multispace0, tag(",")),
            preceded(multispace0, map(identifier, |id| {
                if let AST::Identifier(ref id) = id {
                    id.clone()
                } else {
                    unreachable!()
                }
            }))
        ),
        tag(")")
    )(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, body) = statement(input)?;
    Ok((input, AST::Function {
        name: match name {
            AST::Identifier(id) => id,
            _ => unreachable!(),
        },
        args,
        body: Box::new(body),
    }))
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
        dictionary_literal
    ))(input)?;
    Ok((input, AST::VariableAssign {
        name: match name {
            AST::Identifier(id) => id,
            _ => unreachable!(),
        },
        value: Box::new(value),
    }))
}

// Parsing a statement (includes all possible statements).
pub fn statement(input: &str) -> IResult<&str, AST> {
    alt((
        function,
        return_stmt,
        write_stmt,
        variable_assign,
        if_else,
        coincide,
    ))(input)
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
            Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag)))
        }
    }
}
