use pest::iterators::Pairs;

use super::{
    shader::{Node, SocketValue},
    GraphSignature, Signature, Type,
};

use crate::shader::shader::{GraphInput, InSocket};

use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc, str::FromStr};

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

pub type PResult<T> = Result<T, self::Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Parsing(pest::error::Error<Rule>),
    Code(CodeError, Section),
}

#[derive(Debug, Clone)]
pub enum CodeError {
    Redefinition(String),
    Undefined(String),
    SignatureMismatch(Signature, Signature),
    Type(String, String),
}

#[derive(Debug, Clone)]
pub enum Section {
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

#[derive(Debug)]
struct Import {
    name: String,
    signature: Signature,
}

/// Constructs a [GraphInput] from the eray code passed as input
pub fn parse_shader(eray: &str, loaded: &mut HashMap<String, Rc<RefCell<Node>>>) -> PResult<()> {
    let mut pairs = SParser::parse(Rule::program, eray).map_err(|err| Error::Parsing(err))?;

    let program = pairs.next().unwrap();
    recursive_print(Some(&program), 0);
    parse_program(program, loaded)?;

    Ok(())
}

fn parse_program(
    program: Pair<Rule>,
    loaded: &mut HashMap<String, Rc<RefCell<Node>>>,
) -> PResult<()> {
    let mut inner = program.into_inner();

    let signature = dbg!(parse_signature(inner.next().unwrap())?);
    let imports = dbg!(parse_imports(inner.next().unwrap(), loaded)?);
    // let mut nodes =
    // dbg!(parse_nodes(inner.next().unwrap())?);
    // parse_links(inner.next().unwrap(), &mut nodes)?;

    Ok(())
}

fn parse_imports(
    imports: Pair<Rule>,
    loaded: &mut HashMap<String, Rc<RefCell<Node>>>,
) -> PResult<Vec<Import>> {
    Ok(imports
        .into_inner()
        .map(|import| {
            parse_import(dbg!(import)).map(|import| {
                // Check that the required node has been loaded
                if let Some(loaded) = loaded.get(&import.name) {
                    if import.signature == loaded.borrow().signature() {
                        return Ok(import);
                    }

                    Err(Error::Code(
                        CodeError::SignatureMismatch(import.signature, loaded.borrow().signature()),
                        Section::Imports,
                    ))?;
                }

                Err(Error::Code(
                    CodeError::Undefined(import.name),
                    Section::Imports,
                ))
            })
        })
        .flatten()
        .collect::<PResult<_>>()?)
}

fn parse_import(import: Pair<Rule>) -> PResult<Import> {
    dbg!(&import);
    let mut inner = import.into_inner();

    Ok(Import {
        name: inner.next().unwrap().as_str().to_owned(),
        signature: parse_signature(inner.next().unwrap())?,
    })
}

// fn parse_nodes(nodes: Pair<Rule>) -> PResult<NodeMap> {
//
// }

fn parse_signature(signature: Pair<Rule>) -> PResult<Signature> {
    let mut inner = signature.into_inner();

    let input = dbg!(parse_input(inner.next().unwrap())?);
    let output = dbg!(parse_output(inner.next().unwrap())?);

    Ok(Signature { input, output })
}

fn parse_input(input: Pair<Rule>) -> PResult<HashMap<String, Type>> {
    dbg!(&input);
    let mut res = HashMap::<String, Type>::new();

    for var in input.into_inner() {
        let (id, ty) = parse_var(var);

        if let Some(_) = res.insert(id.clone(), ty) {
            return Err(Error::Code(CodeError::Redefinition(id), Section::Signature));
        }
    }

    Ok(res)
}

fn parse_output(output: Pair<Rule>) -> PResult<HashMap<String, Type>> {
    let mut res = HashMap::<String, Type>::new();

    for var in output.into_inner() {
        let (id, ty) = parse_var(var);

        if let Some(_) = res.insert(id.clone(), ty) {
            return Err(Error::Code(CodeError::Redefinition(id), Section::Signature));
        }
    }

    Ok(res)
}

fn parse_vars(vars: &mut Pairs<Rule>) -> PResult<Vec<(String, Type)>> {
    let mut res = Vec::new();

    dbg!(vars).for_each(|var| res.push(parse_var(var)));

    Ok(res)
}

fn parse_var(var: Pair<Rule>) -> (String, Type) {
    let mut inner = dbg!(var).into_inner();

    (
        inner.next().unwrap().as_str().to_owned(),
        Type::from_str(inner.next().unwrap().as_str()).unwrap(),
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

        assert!(parse_shader(code, &mut HashMap::new()).is_ok());
    }

    #[test]
    fn full_parse() {
        let code = std::fs::read_to_string("nodes/test.eray")
            .expect("Missing `nodes/test.eray` test shader");

        let mut loaded = HashMap::new();
        for node in vec![
            Node::new(
                "add",
                vec![
                    ("lhs".to_owned(), SocketValue::from(Type::Vec3)),
                    ("rhs".to_owned(), SocketValue::from(Type::Color)),
                ]
                .into_iter(),
                vec![("value".to_owned(), SocketValue::from(Type::Value))].into_iter(),
                Box::new(|_input, _output| ()),
            ),
            Node::new(
                "noise",
                vec![
                    ("x".to_owned(), SocketValue::from(Type::Value)),
                    ("y".to_owned(), SocketValue::from(Type::Value)),
                ]
                .into_iter(),
                vec![("value".to_owned(), SocketValue::from(Type::Value))].into_iter(),
                Box::new(|_input, _output| ()),
            ),
        ]
        .into_iter()
        {
            let name = node.borrow().name();
            loaded.insert(name, node);
        }

        let res = parse_shader(code.as_str(), &mut loaded);
        assert!(res.is_ok(), "{res:?}");
    }
}
