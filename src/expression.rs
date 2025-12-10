use std::num::ParseFloatError;

#[derive(Debug, Clone)]
pub enum ExpressionError {
    FailedToParseFloat(ParseFloatError),
    UnknownOperand,
    UnknownOperator,
    InvalidFunction,
    InvalidVariable,
}

pub type ExpressionResult<T> = Result<T, ExpressionError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiate,
    Phase,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Imaginary(f64),
    Real(f64),
    Variable {
        name: String,
        subscript: Option<String>,
    },
    Binop {
        op: BinaryOperator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Bracketed(Box<Expression>),
    Function {
        name: String,
        arguments: Vec<Expression>,
    },
}

fn take_whitespace(input: &str) -> &str {
    input.trim_start()
}

fn take_operand(input: &str) -> ExpressionResult<(Expression, &str)> {
    let input = take_whitespace(input);
    let bytes = input.as_bytes();

    if bytes.is_empty() {
        return Err(ExpressionError::UnknownOperand);
    }

    // Parentheses => bracketed expression
    if bytes[0] == b'(' {
        let mut depth = 1;
        let mut i = 1;
        while i < bytes.len() && depth > 0 {
            if bytes[i] == b'(' {
                depth += 1;
            } else if bytes[i] == b')' {
                depth -= 1;
            }
            i += 1;
        }
        if depth != 0 {
            return Err(ExpressionError::UnknownOperand);
        }
        let inner = &input[1..i - 1];
        let (expr, _) = parse_expr(inner)?;
        let rest = &input[i..];
        return Ok((Expression::Bracketed(Box::new(expr)), rest));
    }

    // Function call or variable
    let mut i = 0;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] > 127)
    {
        i += 1;
    }

    let token = &input[..i];
    let rest = &input[i..];

    let rest = take_whitespace(rest);

    if rest.starts_with('(') {
        let mut depth = 1;
        let mut j = 1;
        while j < rest.len() && depth > 0 {
            match rest.as_bytes()[j] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        if depth != 0 {
            return Err(ExpressionError::InvalidFunction);
        }

        let args_str = &rest[1..j - 1];
        let args: ExpressionResult<Vec<Expression>> = args_str
            .split(',')
            .map(|s| parse_expr(s.trim()).map(|(a, _)| a))
            .collect();

        return Ok((
            Expression::Function {
                name: token.to_string(),
                arguments: args?,
            },
            &rest[j..],
        ));
    }

    if token.contains('_') {
        let parts: Vec<&str> = token.splitn(2, '_').collect();
        return Ok((
            Expression::Variable {
                name: parts[0].to_string(),
                subscript: Some(parts[1].to_string()),
            },
            rest,
        ));
    }

    if let Ok(value) = token.parse::<f64>() {
        return Ok((Expression::Real(value), rest));
    }

    if token.contains('i') || token.contains('j') {
        let input = token.replace('j', "i");
        let parts: Vec<&str> = input.split('i').collect();
        let imag = match &parts[..] {
            [""] => 1.0,
            ["+", ""] => 1.0,
            ["-", ""] => -1.0,
            [num, ""] | ["", num] => num
                .parse::<f64>()
                .map_err(ExpressionError::FailedToParseFloat)?,
            _ => return Err(ExpressionError::UnknownOperand),
        };
        return Ok((Expression::Imaginary(imag), rest));
    }

    // Otherwise variable without subscript
    Ok((
        Expression::Variable {
            name: token.to_string(),
            subscript: None,
        },
        rest,
    ))
}

const OPERATORS: [(&'static str, BinaryOperator); 6] = [
    ("**", BinaryOperator::Exponentiate),
    ("^", BinaryOperator::Exponentiate),
    ("+", BinaryOperator::Add),
    ("-", BinaryOperator::Subtract),
    ("*", BinaryOperator::Multiply),
    ("/", BinaryOperator::Divide),
];

fn take_operator(input: &str) -> ExpressionResult<(BinaryOperator, &str)> {
    let input = input.trim_start();

    for (symbol, op) in OPERATORS {
        if input.starts_with(symbol) {
            let rest = &input[symbol.len()..];
            return Ok((op, rest));
        }
    }

    Err(ExpressionError::UnknownOperator)
}

fn take_binop(input: &str) -> ExpressionResult<(Expression, &str)> {
    let (mut lhs, mut rest) = take_operand(input)?;
    rest = take_whitespace(rest);

    loop {
        let op = match take_operator(rest) {
            Ok((op, r)) => {
                rest = r;
                op
            }
            Err(_) => break,
        };

        rest = take_whitespace(rest);

        let (rhs, r) = take_operand(rest)?;
        rest = r;

        lhs = Expression::Binop {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };

        rest = take_whitespace(rest);
    }

    Ok((lhs, rest))
}

fn precedence(op: BinaryOperator) -> u8 {
    match op {
        BinaryOperator::Phase => 4,
        BinaryOperator::Exponentiate => 3,
        BinaryOperator::Multiply => 2,
        BinaryOperator::Divide => 1,
        BinaryOperator::Add | BinaryOperator::Subtract => 0,
    }
}

fn apply_precedence(expr: Expression) -> Expression {
    use Expression::*;

    match expr {
        Real(_) | Imaginary(_) | Variable { .. } => expr,
        Bracketed(inner) => Bracketed(Box::new(apply_precedence(*inner))),
        Function { name, arguments } => Function {
            name,
            arguments: arguments.into_iter().map(apply_precedence).collect(),
        },
        Binop { op, lhs, rhs } => {
            let lhs = apply_precedence(*lhs);
            let rhs = apply_precedence(*rhs);
            reorder_binop(op, lhs, rhs)
        }
    }
}

fn reorder_binop(op: BinaryOperator, lhs: Expression, rhs: Expression) -> Expression {
    use Expression::*;

    let mut root_op = op;
    let mut left = lhs;
    let mut right = rhs;

    loop {
        if let Binop {
            op: rop,
            lhs: rlhs,
            rhs: rrhs,
        } = right.clone()
        {
            if precedence(rop) < precedence(root_op) {
                right = *rlhs;
                let new_left = Binop {
                    op: root_op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                };
                left = new_left;
                root_op = rop;
                right = *rrhs;
                continue;
            }
        }

        if let Binop {
            op: lop,
            lhs: llhs,
            rhs: lrhs,
        } = left.clone()
        {
            if precedence(lop) < precedence(root_op) {
                left = *lrhs;
                let new_right = Binop {
                    op: root_op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                };
                root_op = lop;
                right = new_right;
                left = *llhs;
                continue;
            }
        }

        break;
    }

    Binop {
        op: root_op,
        lhs: Box::new(left),
        rhs: Box::new(right),
    }
}

pub fn parse_expr(input: &str) -> ExpressionResult<(Expression, &str)> {
    let input = take_whitespace(input);
    let (expr, rest) = take_binop(input)?;
    Ok((apply_precedence(expr), rest))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_expr() {
        let (taken, _) = parse_expr("(1 + 2) + 3 * 4").unwrap();
        println!("{:?}", taken);
    }

    #[test]
    fn test_variable_and_function() {
        let (taken, rest) = parse_expr("max(A_X, B, 2) this is the rest").unwrap();
        println!("{:?}", taken);
        assert_eq!(rest, "this is the rest")
    }
}
