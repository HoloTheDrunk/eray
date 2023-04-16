use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc, str::FromStr};

use crate::shader::shader::{GraphInput, InSocket};

use super::shader::{Node, SocketValue};

use {
    pest::{iterators::Pair, Parser},
    pest_derive::Parser,
};

// macro_rules! debug_pair {
//     ($pair:ident) => {
//         #[cfg(debug_assertions)]
//         {
//             let span = $pair.as_span();
//             println!(
//                 "{}: {:?}: {}",
//                 {
//                     let first = vec.first().unwrap().start_pos().line_col();
//                     let last = vec.last().unwrap().end_pos().line_col();
//
//                     format!("{}:{} -> {}:{}", first.0, first.1, last.0, last.1)
//                 },
//                 $pair.as_rule(),
//                 $pair.as_str()
//             );
//         }
//     };
// }

type PResult<T> = Result<T, self::Error>;

#[derive(Debug, Clone)]
struct Error(ErrorType);

#[derive(Debug, Clone)]
enum ErrorType {
    Parsing(pest::error::Error<Rule>),
    Code(CodeError, Section),
}

#[derive(Debug, Clone)]
enum CodeError {
    Redefinition(String),
    Undefined(String),
    Type(String, String),
}

#[derive(Debug, Clone)]
enum Section {
    Unknown,
    Signature,
    Imports,
    Nodes,
    Links,
}

#[derive(Parser)]
#[grammar = "lib/pest/grammar.pest"]
struct SParser;

type NodeMap<'names> = HashMap<&'names str, Rc<RefCell<Node>>>;

fn parse_shader(eray: &str) -> PResult<()> {
    let mut nodes = NodeMap::new();

    let mut pairs =
        SParser::parse(Rule::program, eray).map_err(|err| Error(ErrorType::Parsing(err)))?;

    let program = pairs.next().unwrap();
    recursive_print(Some(&program), 0);
    parse_program(program, &mut nodes)?;

    Ok(())
}

fn parse_program(program: Pair<Rule>, nodes: &mut NodeMap) -> PResult<()> {
    for section in program.into_inner() {
        match section.as_rule() {
            Rule::signature => {
                dbg!(parse_signature(section, nodes)?);
            }
            Rule::defs => todo!(),
            Rule::decs => todo!(),
            Rule::links => todo!(),
            rule => unreachable!("{rule:?}"),
        }
    }

    Ok(())
}

fn parse_signature(
    signature: Pair<Rule>,
    nodes: &mut NodeMap,
) -> PResult<(HashMap<String, GraphInput>, HashMap<String, InSocket>)> {
    let mut inner = signature.into_inner();

    let input = parse_input(inner.next().unwrap())?;
    let output = parse_output(inner.next().unwrap())?;

    Ok((input, output))
}

fn parse_input(input: Pair<Rule>) -> PResult<HashMap<String, GraphInput>> {
    let mut res = HashMap::<String, GraphInput>::new();

    for var in input.into_inner() {
        let (id, ty) = parse_var(var);

        if let Some(_) = res.insert(id.clone(), GraphInput::new(id.clone(), ty)) {
            return Err(Error(ErrorType::Code(
                CodeError::Redefinition(id),
                Section::Signature,
            )));
        }
    }

    Ok(res)
}

fn parse_output(output: Pair<Rule>) -> PResult<HashMap<String, InSocket>> {
    let mut res = HashMap::<String, InSocket>::new();

    for var in output.into_inner() {
        let (id, ty) = parse_var(var);

        if let Some(_) = res.insert(id.clone(), InSocket::new(id.clone(), ty)) {
            return Err(Error(ErrorType::Code(
                CodeError::Redefinition(id),
                Section::Signature,
            )));
        }
    }

    Ok(res)
}

fn parse_var(var: Pair<Rule>) -> (String, SocketValue) {
    let mut inner = var.into_inner();

    (
        inner.next().unwrap().as_str().to_owned(),
        SocketValue::from_str(inner.next().unwrap().as_str()).unwrap(),
    )
}

fn recursive_print(cur: Option<&Pair<Rule>>, depth: u8) {
    if let Some(node) = cur {
        let rule = node.as_rule();

        let indent = (0..depth)
            .map(|_| "\x1b[32m|   \x1b[0m")
            .collect::<String>();

        println!(
            "{}\x1b[1;33m{:?}\x1b[0m:'{}'",
            indent,
            rule,
            node.as_span()
                .as_str()
                .lines()
                .map(|line| line.trim())
                .collect::<String>()
        );

        for pair in node.clone().into_inner() {
            recursive_print(Some(&pair), depth + 1);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn signature_parse() {
        let code = "|a: Value| -> (a: Value)";

        assert!(parse_shader(code).is_ok())
    }
}
