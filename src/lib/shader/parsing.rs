//! .eray input parsing.
#![allow(unused)]

use super::{
    graph::{
        Graph, ImportedNode, Name, Node, NodeId, SocketRef, SocketType, SocketValue, Unvalidated,
    },
    shader::Side,
    Signature,
};

use crate::{image::Image, shader::graph, ssref, vector::Vector};

use std::{collections::HashMap, fmt::Debug, str::FromStr};

use {
    pest::{error::LineColLocation, iterators::Pair, Parser, Position},
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

macro_rules! match_rule {
    { $pair:ident : $($rule:ident => $action:expr),* $(,)?} => {
        match $pair.as_rule() {
            $(
                Rule::$rule => $action,
            )*
            _ => unreachable!("While matching {:?}", $pair.as_rule()),
        }
    }
}

/// Parsing result.
pub type PResult<T> = Result<T, self::Error>;

#[derive(Debug, Clone, thiserror::Error)]
#[error("Encountered an error while parsing at {line:?}: {kind}")]
/// Parsing error.
pub struct Error {
    kind: ErrorKind,
    line: LineColLocation,
}

impl Error {
    fn new(kind: ErrorKind, line: LineColLocation) -> Self {
        Self { kind, line }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
/// Type of parsing [Error]
pub enum ErrorKind {
    #[error("Pest parsing error: {0}")]
    /// Error during Pest parsing, input is grammatically wrong.
    Parsing(pest::error::Error<Rule>),

    #[error("Code error in {section:?} section: {r#type}")]
    /// Logic error with the input's code.
    Code {
        /// Type of error.
        r#type: CodeError,
        /// File section where the error happened.
        section: Section,
    },
}

#[derive(Debug, Clone, thiserror::Error)]
/// Logic error within the input graph code.
pub enum CodeError {
    #[error("Redefinition of {0}")]
    /// Redefined item (import / node).
    Redefinition(String),

    #[error("Undefined identifier {got}{}{}",
        guess.as_ref().map_or(".".to_string(), |v| format!(", did you mean {v}?")),
        if let UndefinedError::Undefined = variant {
            "".to_string()
        } else {
            format!(" Details: {variant}")
        }
    )]
    /// Use of an undeclared identifier.
    Undefined {
        /// Parsed name.
        got: String,
        /// Lexically closest identifier in the related memory.
        guess: Option<String>,
        /// Subtype providing additional precisions on the error.
        variant: UndefinedError,
    },

    #[error("Import signature mismatch on {}: {1:?} vs {2:?}", .0.to_string())]
    /// Mismatch between two [Signatures](Signature) when importing a [Node].
    SignatureMismatch(Name, Signature, Signature),

    #[error("Socket type mismatc: {0:?} vs {1:?}")]
    /// Mismatch between two sockets' types.
    SocketType(SocketType, SocketType),

    #[error("Trying to link two inputs or outputs together")]
    /// Trying to link two inputs or two outputs.
    SideMismatch,
}

#[derive(Debug, Clone, thiserror::Error)]
/// Detail regarding an undefined identifier.
pub enum UndefinedError {
    #[error("Undefined identifier")]
    /// Identifier is simply undefined.
    Undefined,

    #[error("Unimported identifier {}", name.to_string())]
    /// Name is undefined but is available in the loaded nodes.
    NotImported {
        #[allow(missing_docs)]
        name: Name,
    },

    #[allow(missing_docs)]
    #[error("No signature overload matching {signature:?} for {}", name.to_string())]
    NoSignatureOverload { name: Name, signature: Signature },
}

#[derive(Debug, Clone)]
/// The .eray input section where something happened.
pub enum Section {
    /// Section is unclear or could be multiple.
    Unknown,
    /// Graph type [Signature].
    Signature,
    /// Loaded [Node] importing.
    Imports,
    /// [Node] declarations.
    Nodes,
    /// [Node]-[Node] and [Graph]-[Node] link declarations.
    Links,
}

#[derive(Parser)]
#[grammar = "lib/pest/grammar.pest"]
struct SParser;

#[derive(Clone, Debug)]
struct Import {
    alias: Name,
    name: Name,
    signature: Signature,
}

#[derive(Clone, Debug)]
struct Link {
    prev: LinkSide,
    next: LinkSide,
}

#[derive(Clone, Debug)]
enum LinkSide {
    NodeSocket(NodeId, Name),
    GraphSocket(Name),
    // Value(SocketValue),
}

/// Constructs a [Graph] from the eray code passed as input
pub fn parse_shader(
    eray: &str,
    loaded: &mut HashMap<Name, Vec<ImportedNode<Unvalidated>>>,
) -> PResult<Graph<Unvalidated>> {
    let mut pairs = SParser::parse(Rule::program, eray)
        .map_err(|err| Error::new(ErrorKind::Parsing(err.clone()), err.line_col))?;

    let program = pairs.next().unwrap();
    recursive_print(Some(&program), 0);
    parse_program(program, loaded)
}

fn parse_program(
    program: Pair<Rule>,
    loaded: &mut HashMap<Name, Vec<ImportedNode<Unvalidated>>>,
) -> PResult<Graph<Unvalidated>> {
    let mut inner = program.into_inner();

    let signature = parse_signature(inner.next().unwrap())?;
    let imports = parse_imports(inner.next().unwrap(), loaded)?;
    let mut nodes = parse_nodes(inner.next().unwrap(), loaded, &imports)?;
    let out_links = parse_links(inner.next().unwrap(), &signature, &mut nodes)?;

    let mut graph = graph::Graph {
        inputs: signature
            .input
            .into_iter()
            .map(|(name, socket_type)| (name, socket_type.into()))
            .collect(),
        outputs: signature
            .output
            .into_iter()
            .map(|(name, socket_type)| (name, (None, socket_type.into())))
            .collect(),
        nodes,
        state: std::marker::PhantomData,
    };

    for (name, socket_ref) in out_links.into_iter() {
        let Some((opt, val)) = graph.outputs.get_mut(&name) else {unreachable!()};
        opt.replace(socket_ref);
    }

    Ok(graph)
}

fn lcl_from_bounds((start, end): (Position, Position)) -> LineColLocation {
    LineColLocation::Span(start.line_col(), end.line_col())
}

fn get_loaded(
    loaded: &HashMap<Name, Vec<ImportedNode<Unvalidated>>>,
    name: &Name,
    signature: &Signature,
    rule: &Pair<'_, Rule>,
    section: &Section,
) -> PResult<ImportedNode<Unvalidated>> {
    let err = |undef: UndefinedError| {
        Error::new(
            ErrorKind::Code {
                r#type: CodeError::Undefined {
                    got: name.into(),
                    guess: Some(name.into()),
                    variant: undef,
                },
                section: section.clone(),
            },
            lcl_from_bounds(rule.as_span().split()),
        )
    };

    Ok(loaded
        .get(name)
        .ok_or_else(|| err(UndefinedError::Undefined))?
        .iter()
        .find(|&loaded| &loaded.signature() == signature)
        .ok_or_else(|| {
            err(UndefinedError::NoSignatureOverload {
                name: name.clone(),
                signature: signature.clone(),
            })
        })?
        .clone())
}

fn parse_imports(
    imports: Pair<Rule>,
    loaded: &mut HashMap<Name, Vec<ImportedNode<Unvalidated>>>,
) -> PResult<Vec<Import>> {
    imports
        .into_inner()
        .flat_map(|import| {
            parse_import(import.clone()).map(|parsed| {
                // Among the loaded nodes with the same name, is there one with the correct signature?
                get_loaded(
                    loaded,
                    &parsed.name,
                    &parsed.signature,
                    &import,
                    &Section::Imports,
                )?;

                Ok(parsed)
            })
        })
        .collect::<PResult<_>>()
}

fn parse_import(import: Pair<Rule>) -> PResult<Import> {
    let mut inner = import.into_inner();

    Ok(Import {
        alias: inner.next().unwrap().as_str().into(),
        name: inner.next().unwrap().as_str().into(),
        signature: parse_signature(inner.next().unwrap())?,
    })
}

fn parse_nodes(
    nodes: Pair<Rule>,
    loaded: &HashMap<Name, Vec<ImportedNode<Unvalidated>>>,
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
    loaded: &HashMap<Name, Vec<ImportedNode<Unvalidated>>>,
    imports: &Vec<Import>,
) -> PResult<(NodeId, Node<Unvalidated>)> {
    let mut inner = node.clone().into_inner();

    // Name of the node being declared
    let id = inner.next().unwrap().as_str();

    // Alias of the desired import
    let node_ref = inner.next().unwrap().as_str();
    let name = Name::from(node_ref);

    // Check that that custom node has been imported
    let import = imports
        .iter()
        .find(|&import| import.alias == node_ref.into())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::Code {
                    r#type: CodeError::Undefined {
                        got: node_ref.to_owned(),
                        guess: loaded
                            .contains_key(&name)
                            .then(|| format!("{:?}", loaded.get(&name).unwrap())),
                        variant: UndefinedError::NotImported { name: id.into() },
                    },
                    section: Section::Nodes,
                },
                lcl_from_bounds(node.as_span().split()),
            )
        })?;

    let imported_node = get_loaded(
        loaded,
        &import.name,
        &import.signature,
        &node,
        &Section::Nodes,
    )?;

    Ok((id.into(), Node::Imported(imported_node)))
}

fn parse_links(
    links: Pair<Rule>,
    graph_signature: &Signature,
    nodes: &mut HashMap<NodeId, Node<Unvalidated>>,
) -> PResult<Vec<(Name, SocketRef)>> {
    links
        .into_inner()
        .flat_map(|link| {
            parse_link(link.clone(), graph_signature, nodes)
                .transpose()
                .map(|res| (link, res))
        })
        // .collect::<PResult<Vec<Vec<Link>>>>()
        // .map(|vvec| vvec.into_iter().flatten().collect())
        .map(|(link, res)| {
            res.map(|(name, socket_ref)| {
                if graph_signature.output.get(&name).is_none() {
                    Err(Error::new(
                        ErrorKind::Code {
                            r#type: CodeError::Undefined {
                                got: format!("@OUT.{}", name.to_string()),
                                guess: None,
                                variant: UndefinedError::Undefined,
                            },
                            section: Section::Links,
                        },
                        lcl_from_bounds(link.as_span().split()),
                    ))
                } else {
                    Ok((name, socket_ref))
                }
            })?
        })
        .collect::<PResult<Vec<(Name, SocketRef)>>>()
}

/// Only returns Some if the link's RHS was a graph output socket.
fn parse_link(
    link: Pair<Rule>,
    graph_signature: &Signature,
    nodes: &mut HashMap<NodeId, Node<Unvalidated>>,
    // ) -> PResult<Vec<Link>> {
) -> PResult<Option<(Name, SocketRef)>> {
    let mut inner = link.clone().into_inner();

    let (lhs, rhs) = (inner.next().unwrap(), inner.next().unwrap());

    let (lhs_link, lhs_type) = match_rule! {
        lhs:
            // expr => parse_expr(lhs, graph_signature, nodes),
            field => parse_field(lhs, graph_signature, nodes, &Side::Input),
    }?;

    let (rhs_link, rhs_type) = match_rule! {
        rhs:
            field => parse_field(rhs, graph_signature, nodes, &Side::Output),
    }?;

    match rhs_link {
        LinkSide::NodeSocket(id, name) => nodes
            .get_mut(&id)
            .ok_or(Error::new(
                ErrorKind::Code {
                    r#type: CodeError::Undefined {
                        got: id.to_string(),
                        guess: None,
                        variant: UndefinedError::Undefined,
                    },
                    section: Section::Links,
                },
                lcl_from_bounds(link.as_span().split()),
            ))?
            .set_input(
                &name,
                match lhs_link.clone() {
                    LinkSide::NodeSocket(id, name) => ssref!(node id => name),
                    LinkSide::GraphSocket(name) => ssref!(graph name),
                },
            )
            .map_err(|err| {
                Error::new(
                    ErrorKind::Code {
                        r#type: CodeError::Undefined {
                            got: match lhs_link {
                                LinkSide::NodeSocket(id, name) => {
                                    format!("{}.{}", id.to_string(), name.to_string())
                                }
                                LinkSide::GraphSocket(name) => name.to_string(),
                            },
                            guess: todo!(),
                            variant: todo!(),
                        },
                        section: todo!(),
                    },
                    lcl_from_bounds(link.as_span().split()),
                )
            })?,
        LinkSide::GraphSocket(name) => {
            return Ok(match lhs_link.clone() {
                LinkSide::NodeSocket(id, name) => ssref!(node id => name),
                LinkSide::GraphSocket(name) => ssref!(graph name),
            }
            .map(|socket_ref| (name, socket_ref)))
        }
    };

    Ok(None)
}

/// # Example
/// ```eray
/// expr:  Color(1, 1, 1)
/// field: Color(@IN.vec3)
/// ```
// fn parse_expr(
//     expr: Pair<Rule>,
//     graph_signature: &Signature,
//     nodes: &mut HashMap<NodeId, Node<Unvalidated>>,
// ) -> PResult<(LinkSide, SocketType)> {
//     let mut inner = expr.clone().into_inner();
//
//     // Converted value type target
//     let ty = SocketType::from_str(inner.next().unwrap().as_str()).unwrap();
//
//     let lhs = inner.next().unwrap();
//     let rhs = inner.next().unwrap();
//     match_rule! {
//         lhs:
//             field => {
//                 let (source_link, source_socket_type) = parse_field(lhs, graph_signature, nodes, &Side::Input)?;
//                 let (destination_link, destination_socket_type) = parse_field(rhs, graph_signature, nodes, &Side::Output)?;
//
//
//
//                 todo!();
//             },
//             literal => {
//                 let value = parse_literal(lhs, graph_signature, nodes)?;
//                 todo!();
//             },
//     };
//
//     todo!()
// }

/// Returns:
///     - [SocketValue]: the parsed literal value of the correct type
/// # Example
/// ```eray
/// Color(1, 1, 1)
/// ```
// fn parse_literal(
//     literal: Pair<Rule>,
//     graph_signature: &Signature,
//     nodes: &mut HashMap<NodeId, Node<Unvalidated>>,
// ) -> PResult<SocketValue> {
//     let mut inner = literal.clone().into_inner().next().unwrap();
//
//     let value = match_rule! {
//         inner:
//             value => SocketValue::Value(Some(inner.into_inner().next().unwrap().as_str().parse::<f32>().unwrap())),
//             vector => {
//                 let values = inner
//                     .into_inner()
//                     .map(|number| number.as_str().parse::<f32>())
//                     .collect::<Result<Vec<f32>, _>>()
//                     .unwrap();
//
//                 let vector = Vector::new(values[0], values[1], values[2]);
//
//                 SocketValue::Vec3(Some(vector))
//             },
//     };
//
//     // let duplicate_count = nodes
//     //     .keys()
//     //     .filter(|&key| String::from(key).as_str().starts_with("constant"))
//     //     .count();
//
//     Ok(value)
// }

/// Returns:
///     - [LinkSide]: contains information about the linked socket
///     - [SocketType]: type of the socket
/// # Example
/// ```eray
/// Color(@IN.vec3)
/// ```
fn parse_field(
    field: Pair<Rule>,
    graph_signature: &Signature,
    nodes: &HashMap<NodeId, Node<Unvalidated>>,
    side: &Side,
) -> PResult<(LinkSide, SocketType)> {
    let mut inner = field.clone().into_inner();

    let source = inner.next().unwrap();
    let source = match_rule! {
        source:
            id => Ok(Some(NodeId::from(source.as_str()))),
            meta => match (side, source.as_str()) {
                (Side::Input, "@IN") | (Side::Output, "@OUT") => Ok(None),
                _ => Err(Error::new(
                    ErrorKind::Code {
                        r#type: CodeError::SideMismatch,
                        section: Section::Links
                    },
                    lcl_from_bounds(source.as_span().split())
                )),
            },
    }?;

    let socket = Name::from(inner.next().unwrap().as_str());

    let error = |name: String| {
        Error::new(
            ErrorKind::Code {
                r#type: CodeError::Undefined {
                    got: name,
                    guess: None,
                    variant: UndefinedError::Undefined,
                },
                section: Section::Links,
            },
            lcl_from_bounds(field.as_span().split()),
        )
    };

    Ok(match source {
        Some(node_id) => {
            let r#type = nodes
                .get(&node_id)
                .and_then(|node| match side {
                    Side::Input => node.signature().output.get(&socket).cloned(),
                    Side::Output => node.signature().input.get(&socket).cloned(),
                })
                .ok_or_else(|| error(String::from(&node_id)))?;

            (LinkSide::NodeSocket(node_id, socket), r#type)
        }
        None => {
            let r#type = *match side {
                Side::Input => graph_signature
                    .input
                    .get(&socket)
                    .ok_or_else(|| error(format!("@IN.{}", String::from(&socket))))?,
                Side::Output => graph_signature
                    .output
                    .get(&socket)
                    .ok_or_else(|| error(format!("@OUT.{}", String::from(&socket))))?,
            };

            (LinkSide::GraphSocket(socket), r#type)
        }
    })
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

        if res.insert(id.as_str().into(), ty).is_some() {
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

        if res.insert(id.as_str().into(), ty).is_some() {
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
    use crate::{node, shader::graph, ssref};

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
            vec![
                // Value + Value
                ImportedNode::from((
                    "add",
                    graph! {
                        inputs:
                            "lhs": SocketType::Value.into(),
                            "rhs": SocketType::Value.into(),
                        nodes:
                            "inner": node! {
                                inputs:
                                    "lhs": (ssref!(graph "lhs"), SocketType::Vec3),
                                    "rhs": (ssref!(graph "rhs"), SocketType::Color),
                                outputs:
                                    "value": SocketType::Value.into(),
                            },
                        outputs:
                            "value": (ssref!(node "inner" "value"), SocketType::Value.into())
                    },
                )),
                // Value + Color
                ImportedNode::from((
                    "add",
                    graph! {
                        inputs:
                            "lhs": SocketType::Value.into(),
                            "rhs": SocketType::Color.into(),
                        nodes:
                            "inner": node! {
                                inputs:
                                    "lhs": (ssref!(graph "lhs"), SocketType::Vec3),
                                    "rhs": (ssref!(graph "rhs"), SocketType::Color),
                                outputs:
                                    "value": SocketType::Value.into(),
                            },
                        outputs:
                            "value": (ssref!(node "inner" "value"), SocketType::Value.into())
                    },
                )),
            ],
            vec![ImportedNode::from((
                "noise",
                graph! {
                    inputs:
                        "x": SocketType::Value.into(),
                        "y": SocketType::Value.into(),
                    nodes:
                        "inner": node! {
                            inputs:
                                "x": (ssref!(graph "x"), SocketType::Value),
                                "y": (ssref!(graph "y"), SocketType::Value),
                            outputs:
                                "value": SocketType::Value.into(),
                        },
                    outputs:
                        "value": (ssref!(node "inner" "value"), SocketType::Value.into())
                },
            ))],
        ]
        .into_iter()
        .map(|vec| (vec[0].name().clone(), vec))
        .collect();

        let res = parse_shader(code.as_str(), &mut loaded);
        assert!(res.is_ok(), "{res:?}");
    }
}
