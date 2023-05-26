use pest::{error::LineColLocation, iterators::Pairs, Position, Span};

use super::{
    graph::{node, Graph, ImportedNode, Name, Node, NodeId, SocketType, Unvalidated},
    Signature,
};

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
pub struct Error {
    kind: ErrorKind,
    line: LineColLocation,
}

impl Error {
    fn new(kind: ErrorKind, line: LineColLocation) -> Self {
        Self { kind, line }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Parsing(pest::error::Error<Rule>),
    Code { r#type: CodeError, section: Section },
}

#[derive(Debug, Clone)]
pub enum CodeError {
    Redefinition(String),
    Undefined {
        got: String,
        guess: Option<String>,
        variant: UndefinedError,
    },
    SignatureMismatch(Name, Signature, Signature),
    SocketType(SocketType, SocketType),
}

#[derive(Debug, Clone)]
pub enum UndefinedError {
    Undefined,
    NotImported { node_name: String },
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

#[derive(Debug)]
// TODO: Maybe remove and just load node directly
struct Import {
    name: Name,
    signature: Signature,
}

/// Constructs a [Graph] from the eray code passed as input
pub fn parse_shader(
    eray: &str,
    loaded: &mut HashMap<Name, ImportedNode<Unvalidated>>,
) -> PResult<Graph<Unvalidated>> {
    let mut pairs = SParser::parse(Rule::program, eray)
        .map_err(|err| Error::new(ErrorKind::Parsing(err.clone()), err.line_col))?;

    let program = pairs.next().unwrap();
    recursive_print(Some(&program), 0);
    parse_program(program, loaded)?;

    todo!("Finish .eray graph parsing")
}

fn parse_program(
    program: Pair<Rule>,
    loaded: &mut HashMap<Name, ImportedNode<Unvalidated>>,
) -> PResult<()> {
    let mut inner = program.into_inner();

    let signature = parse_signature(inner.next().unwrap())?;
    let imports = parse_imports(inner.next().unwrap(), loaded)?;
    let mut nodes = parse_nodes(inner.next().unwrap(), loaded, &imports)?;
    // parse_links(inner.next().unwrap(), &mut nodes)?;

    Ok(())
}

fn lcl_from_bounds((start, end): (Position, Position)) -> LineColLocation {
    LineColLocation::Span(start.line_col(), end.line_col())
}

fn parse_imports(
    imports: Pair<Rule>,
    loaded: &mut HashMap<Name, ImportedNode<Unvalidated>>,
) -> PResult<Vec<Import>> {
    Ok(imports
        .into_inner()
        .map(|import| {
            parse_import(import.clone()).map(|parsed| {
                // Check that the required node has been loaded
                if let Some(loaded) = loaded.get(&parsed.name) {
                    if parsed.signature == loaded.signature() {
                        return Ok(parsed);
                    }

                    return Err(Error::new(
                        ErrorKind::Code {
                            r#type: CodeError::SignatureMismatch(
                                parsed.name,
                                parsed.signature,
                                loaded.signature(),
                            ),
                            section: Section::Imports,
                        },
                        lcl_from_bounds(import.as_span().split()),
                    ));
                }

                Err(Error::new(
                    ErrorKind::Code {
                        r#type: CodeError::Undefined {
                            got: String::from(&parsed.name),
                            guess: None,
                            variant: UndefinedError::Undefined,
                        },
                        section: Section::Imports,
                    },
                    lcl_from_bounds(import.as_span().split()),
                ))
            })
        })
        .flatten()
        .collect::<PResult<_>>()?)
}

fn parse_import(import: Pair<Rule>) -> PResult<Import> {
    let mut inner = import.into_inner();

    Ok(Import {
        name: inner.next().unwrap().as_str().into(),
        signature: parse_signature(inner.next().unwrap())?,
    })
}

fn parse_nodes(
    nodes: Pair<Rule>,
    loaded: &HashMap<Name, ImportedNode<Unvalidated>>,
    imports: &Vec<Import>,
) -> PResult<HashMap<NodeId, Node<Unvalidated>>> {
    // XXX
    // let mut res = HashMap::new();
    // for node in nodes.into_inner() {
    //     let (id, node) = parse_node(node, loaded, imports)?;
    //     res.insert(id, node);
    // }
    //
    // Ok(res)
    nodes
        .into_inner()
        .map(|node| parse_node(node, loaded, imports))
        .collect()
}

fn parse_node(
    node: Pair<Rule>,
    loaded: &HashMap<Name, ImportedNode<Unvalidated>>,
    imports: &Vec<Import>,
) -> PResult<(NodeId, Node<Unvalidated>)> {
    let mut inner = node.clone().into_inner();

    let id = inner.next().unwrap().as_str();
    let node_ref = inner.next().unwrap().as_str();

    if node_ref.chars().next().unwrap() == '$' {
        let name: Name = node_ref.chars().skip(1).collect::<String>().as_str().into();

        // Check that that custom node has been imported
        imports
            .iter()
            .find(|&import| import.name == node_ref.into())
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::Code {
                        r#type: CodeError::Undefined {
                            got: node_ref.to_owned(),
                            guess: loaded
                                .contains_key(&name)
                                .then(|| format!("{:?}", loaded.get(&name).unwrap())),
                            variant: UndefinedError::NotImported {
                                node_name: id.to_owned(),
                            },
                        },
                        section: Section::Nodes,
                    },
                    lcl_from_bounds(node.as_span().split()),
                )
            })?;

        let imported_node = loaded.get(&name).unwrap().clone();

        Ok((id.into(), Node::Imported(imported_node)))
    } else {
        todo!()
    }

    // if let Some(node) = loaded.get(&stripped) {}

    // let signature = imports.iter().find(|&&import| import.name;

    // Ok(Node::new(id.as_str(), ))
}

fn parse_signature(signature: Pair<Rule>) -> PResult<Signature> {
    let mut inner = signature.into_inner();

    let input = parse_input(inner.next().unwrap())?;
    let output = parse_output(inner.next().unwrap())?;

    Ok(Signature { input, output })
}

fn parse_input(input: Pair<Rule>) -> PResult<HashMap<Name, SocketType>> {
    let mut res = HashMap::new();

    for var in input.into_inner() {
        let span = var.as_span();
        let (id, ty) = parse_var(var);

        if let Some(_) = res.insert(id.as_str().into(), ty) {
            return Err(Error::new(
                ErrorKind::Code {
                    r#type: CodeError::Redefinition(id),
                    section: Section::Signature,
                },
                lcl_from_bounds(span.split()),
            ));
        }
    }

    Ok(res)
}

fn parse_output(output: Pair<Rule>) -> PResult<HashMap<Name, SocketType>> {
    let mut res = HashMap::new();

    for var in output.into_inner() {
        let span = var.as_span();
        let (id, ty) = parse_var(var);

        if let Some(_) = res.insert(id.as_str().into(), ty) {
            return Err(Error::new(
                ErrorKind::Code {
                    r#type: CodeError::Redefinition(id),
                    section: Section::Signature,
                },
                lcl_from_bounds(span.split()),
            ));
        }
    }

    Ok(res)
}

fn parse_var(var: Pair<Rule>) -> (String, SocketType) {
    let mut inner = var.into_inner();

    (
        inner.next().unwrap().as_str().to_owned(),
        SocketType::from_str(inner.next().unwrap().as_str()).unwrap(),
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
    use crate::{shader::graph, ssref};

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

        let mut loaded = vec![
            ImportedNode::from((
                "add",
                graph! {
                    inputs:
                        "lhs": SocketType::Vec3.into(),
                        "rhs": SocketType::Color.into(),
                    nodes:
                        "inner": node! {
                            inputs:
                                "lhs": (ssref!(graph "lhs"), SocketType::Vec3.into()),
                                "rhs": (ssref!(graph "rhs"), SocketType::Color.into()),
                            outputs:
                                "value": SocketType::Value.into(),
                        },
                    outputs:
                        "value": (ssref!(node "inner" "value"), SocketType::Value.into())
                },
            )),
            ImportedNode::from((
                "noise",
                graph! {
                    inputs:
                        "x": SocketType::Value.into(),
                        "y": SocketType::Value.into(),
                    nodes:
                        "inner": node! {
                            inputs:
                                "x": (ssref!(graph "x"), SocketType::Value.into()),
                                "y": (ssref!(graph "y"), SocketType::Value.into()),
                            outputs:
                                "value": SocketType::Value.into(),
                        },
                    outputs:
                        "value": (ssref!(node "inner" "value"), SocketType::Value.into())
                },
            )),
        ]
        .into_iter()
        .map(|node| (node.name().clone(), node))
        .collect();

        let res = parse_shader(code.as_str(), &mut loaded);
        assert!(res.is_ok(), "{res:?}");
    }
}
