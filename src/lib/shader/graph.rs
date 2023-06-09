//! Flat (non-recursive) [Graph] data structure implementation.

use std::{
    collections::{HashMap, VecDeque},
    convert::AsRef,
    fmt::Debug,
    marker::PhantomData,
    str::FromStr,
};

use super::{shader::Shader, Signature};

use crate::{color::Color, image::Image, vector::Vector};

macro_rules! socket_value {
    { $($(#[$attr:meta])* $name:ident : $type:ty = $default:expr),+ $(,)? } => {
        #[derive(Clone, Debug, PartialEq)]
        /// Possible socket value types.
        pub enum SocketValue {
            $(
                $(#[$attr])*
                $name(Option<$type>)
            ),+
        }

        impl From<SocketType> for SocketValue {
            fn from(value: SocketType) -> Self {
                match value {
                    $(
                        SocketType::$name => Self::$name(None)
                    ),+
                }
            }
        }

        impl AsRef<SocketValue> for SocketValue {
            fn as_ref(&self) -> &SocketValue {
                self
            }
        }

        impl SocketValue {
            /// Check if the socket has a value
            pub fn is_none(&self) -> bool {
                match self {
                    $(
                        SocketValue::$name(opt) => opt.is_none()
                    ),+
                }
            }

            /// Set the contained value to its type's defined default.
            pub fn set_default(&mut self) {
                match self {
                    $(
                        SocketValue::$name(ref mut opt) => *opt = Some($default)
                    ),+
                }
            }

            /// Get the contained value, defaulting it beforehand if it is None.
            pub fn or_default(&mut self) -> &SocketValue {
                if self.is_none() {
                    self.set_default();
                }

                self
            }
        }

        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        /// Possible socket types.
        pub enum SocketType {
            #[default]
            $(
                $(#[$attr])*
                $name
            ),+
        }

        impl<T: AsRef<SocketValue>> From<T> for SocketType {
            fn from(value: T) -> Self {
                match value.as_ref() {
                    $(
                        SocketValue::$name(_) => Self::$name
                    ),+
                }
            }
        }

        impl FromStr for SocketType {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    $(
                        stringify!($name) => Self::$name
                    ),+,
                    other => Err(format!("Unrecognized socket type `{other}`."))?,
                })
            }
        }
    };
}

socket_value! {
    /// Single value
    Number: f32 = 0.,
    /// Character [String]
    String: String = String::from(""),

    /// Image of floating-point values
    Value: Image<f32> = Image::default(),
    /// Image of [3D vectors](Vec3)
    Vec3: Image<Vector<3, f32>> = Image::default(),
    /// Image of [colors](Color)
    Color: Image<Color> = Image::default(),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
/// Wrapper around [String].
pub struct NodeId(String);
impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
/// Wrapper around [String].
pub struct Name(String);
impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
impl From<&Name> for String {
    fn from(name: &Name) -> Self {
        name.0.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Reference to a [Graph] or [Node] socket.
pub enum SocketRef {
    /// Node [NodeId] and output socket [Name]
    Node(NodeId, Name),
    /// Graph input socket [Name]
    Graph(Name),
}

#[macro_export]
/// Shorthand to reference sockets from the [Graph](Graph) or other [Node](Node)s.
/// # Example
/// ```
/// let graph_socket = sref!(graph "socket_name");
/// assert_eq!(graph_socket, SocketRef::Graph(Name::from("socket_name")));
///
/// let node_socket = sref!(node "node_name" "socket_name");
/// assert_eq!(node_socket, SocketRef::Node(Name::from("node_name"), Name::from("socket_name")));
/// ```
macro_rules! sref {
    (graph $field:literal) => {
        crate::shader::graph::SocketRef::Graph(crate::shader::graph::Name::from($field))
    };

    (node $node:literal $field:literal) => {
        crate::shader::graph::SocketRef::Node(
            crate::shader::graph::NodeId::from($node),
            crate::shader::graph::Name::from($field),
        )
    };
}

#[macro_export]
/// Shorthand to reference sockets from the [Graph]s or other [Node]s, wrapped in an
/// [Option::Some]. Calls [sref] internally so the syntax is the same.
/// # Example
/// ```
/// assert_eq!(ssref!(graph "value"), Some(sref!(graph "value")));
/// ```
macro_rules! ssref {
    ($($tree:tt)+) => {
        Some(crate::shader::graph::sref!($($tree)+))
    };
}

macro_rules! states {
    ($($(#[$attr:meta])* $state:ident),+ $(,)?) => {
        $(
            #[derive(Clone, Debug, Default, PartialEq)]
            $(#[$attr])*
            pub struct $state;
        )+
    };
}
states! {
    /// Minimal checks have been done to make sure the graph is usable.
    Unvalidated,
    /// Checked for cycles, socket connection types, etc...
    Validated,
}

#[derive(Debug, PartialEq)]
/// [Graph] error
pub enum Error {
    /// Detected a cycle on the node with the given [NodeId].
    Cycle(NodeId),
    /// [Shader] returned with an error.
    Shader(super::shader::Error),
}

impl From<super::shader::Error> for Error {
    fn from(value: super::shader::Error) -> Self {
        Self::Shader(value)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Flat graph data structure state machine implementation.
pub struct Graph<State> {
    /// Graph inputs consisting of a [Name] and a value set before calling the [Graph].
    pub inputs: HashMap<Name, SocketValue>,
    /// Graph outputs consisting of a [Name], a reference to a [Graph] input or [Node] output
    /// socket, along with an output value.
    pub outputs: HashMap<Name, (Option<SocketRef>, SocketValue)>,

    /// Mapping of [NodeIds](NodeId) to [Nodes](Node).
    pub nodes: HashMap<NodeId, Node<State>>,

    /// Current state
    pub state: PhantomData<State>,
}

#[macro_export]
/// Instantiate a [Graph] concisely.
/// # Example
/// ```
/// graph! {
///     inputs:
///         "iFac": SocketValue::Number(Some(2.)),
///         "iName": SocketValue::String(None),
///     nodes:
///         "identity": node! {
///             inputs:
///                 "value": (ssref!(graph "iFac"), SocketType::Number),
///             outputs:
///                 "value": SocketType::Number.into();
///             |_inputs, _outputs| Ok(())
///         },
///         "invert": node! {
///             inputs:
///                 "value": (ssref!(node "identity" "value"), SocketType::Number),
///             outputs:
///                 "value": SocketType::Number.into();
///         },
///     outputs:
///         "oFac": (ssref!(node "invert" "value"), SocketValue::Number(None)),
/// };
/// ```
macro_rules! graph {
    { $($field:ident $(: $($name:literal : $value:expr),+)? $(,)?),+ } => {
        Graph {
            $($field: [$($(($name.into(), $value)),+)?].into_iter().collect()),+,
            state: ::std::marker::PhantomData::<crate::shader::graph::Unvalidated>,
        }
    };
}

impl Graph<Unvalidated> {
    /// Check the [unvalidated](Unvalidated) [Graph] for cycles.
    pub fn validate(self) -> Result<Graph<Validated>, Error> {
        for output in self.outputs.iter() {
            let mut path = Vec::<&NodeId>::new();
            let mut next = VecDeque::new();
            next.push_back({
                let (_name, (Some(socket_ref), _value)) = output else {continue};
                match socket_ref {
                    SocketRef::Node(node_id, _name) => Some(node_id),
                    SocketRef::Graph(_name) => None,
                }
            });

            while let Some(Some(id)) = next.pop_front() {
                // Check for cycle in graph
                if path.contains(&id) {
                    return Err(Error::Cycle(id.clone()));
                }

                path.push(id);

                self.nodes.get(id).map(|node| {
                    let inputs = match node {
                        Node::Graph(graph_node) => graph_node.inputs.values(),
                        Node::Imported(imported_node) => imported_node.inputs.values(),
                    };

                    for socket_ref in inputs.map(|(opt, _type)| opt).flatten() {
                        match socket_ref {
                            SocketRef::Node(node_id, _name) => next.push_back(Some(node_id)),
                            // Ignore graph inputs
                            SocketRef::Graph(_name) => continue,
                        }
                    }
                });
            }
        }

        let Self {
            inputs,
            outputs,
            nodes,
            state: _state,
        } = self;

        Ok(Graph {
            inputs,
            outputs,
            nodes: nodes
                .into_iter()
                .map(|(k, v)| Ok((k, v.validate()?)))
                .collect::<Result<_, Error>>()?,
            state: PhantomData::<Validated>,
        })
    }
}

impl Graph<Validated> {
    /// Run graph by computing connected shader nodes recursively.
    /// The final results are contained in the graph's `outputs` hashmap.
    pub fn run(&mut self) -> Result<(), Error> {
        // Dirtily cloning the entire outputs hashmap but it works
        self.outputs = self
            .outputs
            .clone()
            .into_iter()
            .filter(|(_name, (_ref, value))| value.is_none())
            .map(|output| {
                let (name, (socket_ref, mut value)) = output;

                // Unconnected output
                if socket_ref.is_none() {
                    value.set_default();
                    return Ok((name, (socket_ref, value)));
                }

                // Get rid of Option
                let socket_ref = socket_ref.unwrap();
                match &socket_ref {
                    SocketRef::Node(node_id, name) => {
                        // Recurse into node to run it
                        self.run_node(node_id)?;
                        // Get output value of node connected to graph output
                        value = (*self
                            .nodes
                            .get(node_id)
                            .unwrap()
                            .outputs()
                            .get(&name)
                            .unwrap())
                        .clone();
                    }
                    SocketRef::Graph(name) => value = self.inputs.get(&name).unwrap().clone(),
                };

                Ok((name, (Some(socket_ref), value)))
            })
            .collect::<Result<_, Error>>()?;

        Ok(())
    }

    /// Run node by computing its inputs recursively, then computing the contained shader
    fn run_node(&mut self, node_id: &NodeId) -> Result<(), Error> {
        // Skip node if outputs are already computed.
        if self
            .nodes
            .get(node_id)
            .unwrap()
            .outputs()
            .iter()
            .all(|(&_k, &v)| !v.is_none())
        {
            return Ok(());
        }

        let cur = self.nodes.get(node_id).unwrap().clone();

        match cur {
            Node::Graph(cur_inner) => {
                let mut inputs = HashMap::new();

                for (name, (socket_ref, r#type)) in cur_inner.inputs.into_iter() {
                    if let Some(socket_ref) = socket_ref {
                        inputs.insert(
                            name,
                            match socket_ref {
                                SocketRef::Node(id, field) => {
                                    self.run_node(&id)?;
                                    (*self.nodes.get(&id).unwrap().outputs().get(&field).unwrap())
                                        .clone()
                                }
                                SocketRef::Graph(field) => self.inputs.get(&field).unwrap().clone(),
                            },
                        );
                    } else {
                        inputs.insert(name, r#type.into());
                    }
                }

                let Some(Node::Graph(node)) = self.nodes.get_mut(node_id) else {unreachable!()};
                node.shader.call(&inputs, &mut node.outputs)?;
            }
            Node::Imported(cur_inner) => {
                for (name, (socket_ref, r#_type)) in cur_inner.inputs.into_iter() {
                    if let Some(socket_ref) = socket_ref {
                        let value = match socket_ref.clone() {
                            SocketRef::Node(id, field) => {
                                self.run_node(&id)?;
                                (*self.nodes.get(&id).unwrap().outputs().get(&field).unwrap())
                                    .clone()
                            }
                            SocketRef::Graph(field) => self.inputs.get(&field).unwrap().clone(),
                        };

                        let Some(Node::Imported(node)) = self.nodes.get_mut(node_id) else {unreachable!()};
                        node.inner.inputs.insert(name, value);
                    } else {
                        let Some(Node::Imported(node)) = self.nodes.get_mut(node_id) else {unreachable!()};
                        node.inner.inputs.get_mut(&name).unwrap().set_default();
                    }
                }

                let Some(Node::Imported(node)) = self.nodes.get_mut(node_id) else {unreachable!()};
                node.inner.run()?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Default)]
/// A "raw" node with its own [Shader].
pub struct GraphNode {
    /// Node inputs.
    pub inputs: HashMap<Name, (Option<SocketRef>, SocketType)>,
    /// Node outputs.
    pub outputs: HashMap<Name, SocketValue>,

    /// Function to be run, taking the inputs and modifying the output values.
    pub shader: Shader,
}

impl Debug for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("inputs", &self.inputs)
            .field("outputs", &self.outputs)
            .finish_non_exhaustive()
    }
}

impl PartialEq for GraphNode {
    fn eq(&self, other: &Self) -> bool {
        // Ignore shader
        self.inputs == other.inputs && self.outputs == other.outputs
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Imported node, i.e. a sub-graph based on a loaded [Graph].
pub struct ImportedNode<State> {
    name: Name,
    /// Iputs to be passed to the inner [Graph].
    pub inputs: HashMap<Name, (Option<SocketRef>, SocketType)>,
    inner: Graph<State>,
}

impl<State> ImportedNode<State> {
    /// Get the [Name] of the loaded [Graph] used to create this [ImportedNode].
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Get the sub-graph's type signature.
    pub fn signature(&self) -> Signature {
        Signature {
            input: self
                .inputs
                .iter()
                .map(|(name, (_socket_ref, socket_type))| (name.clone(), socket_type.clone()))
                .collect(),

            output: self
                .inner
                .outputs
                .iter()
                .map(|(name, (_socket_ref, value))| (name.clone(), value.clone().into()))
                .collect(),
        }
    }
}

impl<T: AsRef<str>, State> From<(T, Graph<State>)> for ImportedNode<State> {
    fn from((name, inner): (T, Graph<State>)) -> Self {
        Self {
            inputs: inner
                .inputs
                .iter()
                .map(|(name, value)| (name.clone(), (None, value.into())))
                .collect(),
            name: Name::from(name.as_ref()),
            inner,
        }
    }
}

impl ImportedNode<Unvalidated> {
    fn validate(self) -> Result<ImportedNode<Validated>, Error> {
        let ImportedNode {
            name,
            inputs,
            inner,
        } = self;

        Ok(ImportedNode {
            name,
            inputs,
            inner: inner.validate()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
/// A simple graph node or an imported sub-graph node.
pub enum Node<State> {
    /// Single graph node.
    Graph(GraphNode),
    /// Imported sub-graph.
    Imported(ImportedNode<State>),
}

impl<State> Default for Node<State> {
    fn default() -> Self {
        Self::Graph(GraphNode::default())
    }
}

impl Node<Unvalidated> {
    fn validate(self) -> Result<Node<Validated>, Error> {
        Ok(match self {
            Node::Graph(node) => Node::Graph(node),
            Node::Imported(node) => Node::Imported(node.validate()?),
        })
    }
}

impl<State> Node<State> {
    fn inputs(&self) -> &HashMap<Name, (Option<SocketRef>, SocketType)> {
        match self {
            Node::Graph(node) => &node.inputs,
            Node::Imported(node) => &node.inputs,
        }
    }

    fn outputs(&self) -> HashMap<&Name, &SocketValue> {
        match self {
            Node::Graph(node) => node.outputs.iter().collect(),
            Node::Imported(node) => node
                .inner
                .outputs
                .iter()
                .map(|(name, (_socket_ref, value))| (name, value))
                .collect(),
        }
    }

    /// Get the node's (and by extension the shader's) type signature.
    pub fn signature(&self) -> Signature {
        let input = self
            .inputs()
            .iter()
            .map(|(name, (_socket_ref, socket_type))| (name.clone(), socket_type.clone()))
            .collect();

        let output = self
            .outputs()
            .iter()
            .map(|(&name, &value)| (name.clone(), value.clone().into()))
            .collect();

        Signature { input, output }
    }
}

#[macro_export]
/// Instantiate a node concisely
/// # Example
/// ```
/// node! {
///     inputs:
///         "value": ssref!(graph "iFac"),
///     outputs:
///         "value": SocketValue::Number(None);
///     |_inputs, _outputs| ()
/// }
/// ```
/// The shader is optional and will be defaulted if empty.
///
/// See [Shader::new] for an example function.
macro_rules! node {
    ( import $name:literal $imported:expr $(,)?) => {
        crate::shader::graph::Node::Imported(
            $imported
                .get($name)
                .expect(format!("Could not find imported node `{}`. Imported nodes are: {}",
                    $name, $imported.keys().cloned().collect::<Vec<String>>().join(", ")).as_str()).clone()
        )
    };

    ( import $name:literal $imported:expr $(, inputs: $($input:literal : $socket_ref:expr)+)? $(,)?) => {
        crate::shader::graph::Node::Imported({
            let mut res = $imported
                .get($name)
                .expect(format!("Could not find imported node `{}`. Imported nodes are: {}",
                    $name, $imported.keys().cloned().collect::<Vec<String>>().join(", ")).as_str()).clone();

            $(
                $(
                    let len = res.inputs.len();
                    let inputs = res
                        .inputs
                        .keys()
                        .map(String::from)
                        .collect::<Vec<String>>()
                        .join(", ");

                    *res.inputs.get_mut(&$input.into()).expect(
                        format!(
                            "Could not find input `{}` for node `{}`. Node's inputs are: ({}) [{}]",
                            $input, $name, len, inputs
                        )
                        .as_str(),
                    ) = $socket_ref;
                )+
            )?

            res
        })
    };

    { $($field:ident $(: $($o_name:literal : $value:expr),+)?),+; $shader:expr $(,)? } => {
        crate::shader::graph::Node::Graph(crate::shader::graph::GraphNode {
            $($field: [$($(($o_name.into(), $value)),+)?].into_iter().collect()),+,
            shader: crate::shader::shader::Shader::new($shader),
        })
    };

    { $($field:ident $(: $($o_name:literal : $value:expr),+)?),+ $(,)? $(;)? } => {
        crate::shader::graph::Node::Graph(crate::shader::graph::GraphNode {
            $($field: [$($(($o_name.into(), $value)),+)?].into_iter().collect()),+,
            shader: crate::shader::shader::Shader::default(),
        })
    };
}

// Export macros
pub use {graph, node, sref, ssref};

#[cfg(test)]
mod test {
    use super::*;
    use crate::get_sv;

    fn setup_imports() -> HashMap<String, ImportedNode<Unvalidated>> {
        std::iter::once((
            "identity".to_owned(),
            ImportedNode::from((
                "identity",
                graph! {
                    inputs:
                        "value": SocketValue::Number(Some(0.)),
                    nodes:
                        "id": node! {
                            inputs:
                                "value": (None, SocketType::Number),
                            outputs:
                                "value": SocketType::Number.into();
                            |inputs, outputs| {
                                get_sv!(input | inputs . "value" : Number > in_value);
                                get_sv!(output | outputs . "value" : Number > out_value);

                                *out_value.get_or_insert(0.) = in_value.unwrap_or(0.);

                                Ok(())
                            }
                        },
                    outputs:
                        "value": (ssref!(node "id" "value"), SocketType::Number.into()),
                },
            )),
        ))
        .collect()
    }

    #[test]
    fn cycle_detection() {
        let imported = setup_imports();

        let validation_result = graph! {
            inputs,
            nodes:
                "a": node! {
                    import "identity" imported,
                    inputs:
                        "value": (ssref!(node "b" "value"), SocketType::Number),
                },
                "b": node! {
                    import "identity" imported,
                    inputs:
                        "value": (ssref!(node "a" "value"), SocketType::Number),
                },
            outputs:
                "value": (ssref!(node "a" "value"), SocketType::Number.into()),
        }
        .validate();

        let expected = Error::Cycle(NodeId("a".to_owned()));

        assert!(
            validation_result.is_err(),
            "Expected an error, got `{validation_result:?}`"
        );

        assert_eq!(validation_result.unwrap_err(), expected);
    }

    #[test]
    fn macro_validity() {
        let manual = Graph {
            inputs: [
                (Name::from("iFac"), SocketValue::Number(Some(2.))),
                (Name::from("iName"), SocketValue::String(None)),
            ]
            .into_iter()
            .collect(),
            nodes: [
                (
                    NodeId::from("identity"),
                    GraphNode {
                        inputs: std::iter::once((
                            Name::from("value"),
                            (
                                Some(SocketRef::Graph(Name::from("iFac"))),
                                SocketType::Number,
                            ),
                        ))
                        .collect(),
                        outputs: std::iter::once((Name::from("value"), SocketValue::Number(None)))
                            .collect(),
                        ..Default::default()
                    },
                ),
                (
                    NodeId::from("invert"),
                    GraphNode {
                        inputs: std::iter::once((
                            Name::from("value"),
                            (
                                Some(SocketRef::Node(
                                    NodeId::from("identity"),
                                    Name::from("value"),
                                )),
                                SocketType::Number,
                            ),
                        ))
                        .collect(),
                        outputs: [(Name::from("value"), SocketValue::Number(None))]
                            .into_iter()
                            .collect(),
                        ..Default::default()
                    },
                ),
            ]
            .map(|(name, node)| (name, Node::Graph(node)))
            .into_iter()
            .collect(),
            outputs: std::iter::once((
                Name::from("oFac"),
                (
                    Some(SocketRef::Node(NodeId::from("invert"), Name::from("value"))),
                    SocketValue::Number(None),
                ),
            ))
            .collect(),
            state: PhantomData::<Unvalidated>,
        };

        let r#macro = graph! {
            inputs:
                "iFac": SocketValue::Number(Some(2.)),
                "iName": SocketValue::String(None),
            nodes:
                "identity": node! {
                    inputs:
                        "value": (ssref!(graph "iFac"), SocketType::Number),
                    outputs:
                        "value": SocketType::Number.into()
                },
                "invert": node! {
                    inputs:
                        "value": (ssref!(node "identity" "value"), SocketType::Number),
                    outputs:
                        "value": SocketType::Number.into();
                },
            outputs:
                "oFac": (ssref!(node "invert" "value"), SocketValue::Number(None)),
        };

        assert_eq!(manual, r#macro);
    }
}
