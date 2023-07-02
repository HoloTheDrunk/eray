//! Flat (non-recursive) [Graph] data structure implementation.

use super::{
    shader::{Shader, Side},
    Signature,
};

use crate::{color::Color, image::{Image, Convertible}, vector::Vector};

use std::{
    collections::{HashMap, VecDeque},
    convert::AsRef,
    fmt::Debug,
    marker::PhantomData,
    str::FromStr,
    string::ToString,
};

use paste::paste;

macro_rules! socket_value {
    { $($(#[$attr:meta])* $name:ident : $type:ty = $default:expr),+ $(,)? } => {
        paste! {
            #[allow(unused)]
            #[derive(Clone, Debug, PartialEq)]
            /// Possible socket value types.
            pub enum SocketValue {
                $(
                    $(#[$attr])*
                    $name(Option<$type>),

                    #[doc = concat!("Image of [", stringify!($name), "](", stringify!($name), ") values")]
                    [<I  $name>](Option<Image<$type>>),
                )+
            }

            impl From<SocketType> for SocketValue {
                fn from(value: SocketType) -> Self {
                    match value {
                        $(
                            SocketType::$name => Self::$name(None),
                            SocketType::[<I  $name>] => Self::[<I  $name>](None),
                        )+
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
                            SocketValue::$name(opt) => opt.is_none(),
                            SocketValue::[<I  $name>](opt) => opt.is_none(),
                        )+
                    }
                }

                /// Set the contained value to its type's defined default.
                pub fn set_default(&mut self) {
                    match self {
                        $(
                            SocketValue::$name(ref mut opt) => *opt = Some($default),
                            SocketValue::[<I  $name>](ref mut opt) => *opt = Some(Image::default()),
                        )+
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

            #[allow(unused)]
            #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
            /// Possible socket types.
            pub enum SocketType {
                #[default]
                $(
                    $(#[$attr])*
                    $name,

                    #[doc = concat!("Image of [", stringify!($name), "](", stringify!($name), ") values")]
                    [<I  $name>],
                )+
            }

            impl<T: AsRef<SocketValue>> From<T> for SocketType {
                fn from(value: T) -> Self {
                    match value.as_ref() {
                        $(
                            SocketValue::$name(_) => Self::$name,
                            SocketValue::[<I  $name>](_) => Self::[<I $name>],
                        )+
                    }
                }
            }

            impl FromStr for SocketType {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Ok(match s {
                        $(
                            stringify!($name) => Self::$name,
                            stringify!([<I  $name>]) => Self::[<I  $name>],
                        )+
                        other => Err(format!("Unrecognized socket type `{other}`."))?,
                    })
                }
            }
        }
    };
}

macro_rules! socket_conversions {
    ($($src:ident => $($dst:ident by $method:path)|+),+ $(,)?) => {paste!{
        impl SocketValue {
            /// Attempt conversion between two socket values.
            pub fn try_convert(self, target: SocketType) -> Result<Self, ()> {
                let err = Err(());

                Ok(match self {
                    $(
                        SocketValue::$src(opt) => match target {
                            $(
                                SocketType::$dst => SocketValue::$dst(opt.map($method)),
                            )+
                            #[allow(unreachable_patterns)]
                            _ => err?,
                        },

                        SocketValue::[<I $src>](opt) => match target {
                            $(
                                SocketType::[<I $dst>] => SocketValue::[<I $dst>](opt.map(|img| Image::convert_image(img, $method))),
                            )+
                            #[allow(unreachable_patterns)]
                            _ => err?,
                        }
                    )+
                    #[allow(unreachable_patterns)]
                    _ => err?,
                })
            }
        }
    }};
}

socket_value! {
    /// Single value
    Value: f32 = 0.,
    /// 2D vector
    Vec2: Vector<2, f32> = Vector::default(),
    /// 3D vector
    Vec3: Vector<3, f32> = Vector::default(),
    /// 3-channel color
    Color: Color = Color::default(),
}

socket_conversions! {
    Value => Vec2 by Into::into | Vec3 by Into::into | Color by Into::into,
    Vec2 => Value by Into::into,
    Vec3 => Value by Into::into | Color by Into::into,
    Color => Value by Into::into | Vec3 by Into::into,
}

// impl SocketValue {
//     /// Attempt conversion between two socket values.
//     pub fn try_convert(self, target: SocketType) -> Result<Self, ()> {
//         let err = Err(());
//
//         Ok(match self {
//             SocketValue::Value(opt) => match target {
//                 SocketType::Vec2 => SocketValue::Vec2(opt.map(From::from)),
//                 SocketType::Vec3 => SocketValue::Vec3(opt.map(From::from)),
//                 SocketType::Color => SocketValue::Color(opt.map(From::from)),
//                 _ => err?,
//             },
//             SocketValue::IValue(opt) => match target {
//                 SocketType::IVec2 => SocketValue::IVec2(opt.map(Image::convert_image)),
//                 SocketType::IVec3 => SocketValue::IVec3(opt.map(Image::convert_image)),
//                 SocketType::IColor => SocketValue::IColor(opt.map(Image::convert_image)),
//                 _ => err?,
//             },
//
//             SocketValue::Vec3(opt) => todo!(),
//             SocketValue::IVec3(opt) => todo!(),
//
//             SocketValue::Color(opt) => todo!(),
//             SocketValue::IColor(opt) => todo!(),
//
//             _ => err?,
//         })
//     }
// }

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
/// Wrapper around [String].
pub struct NodeId(String);
impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}
impl From<&NodeId> for String {
    fn from(id: &NodeId) -> Self {
        id.0.clone()
    }
}
impl ToString for NodeId {
    fn to_string(&self) -> String {
        self.0.clone()
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
impl ToString for Name {
    fn to_string(&self) -> String {
        self.0.clone()
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
/// use eray::{sref, shader::graph::{SocketRef, Graph, Name, NodeId}};
///
/// let graph_socket = sref!(graph "socket_name");
/// assert_eq!(graph_socket, SocketRef::Graph(Name::from("socket_name")));
///
/// let node_socket = sref!(node "node_name" "socket_name");
/// assert_eq!(node_socket, SocketRef::Node(NodeId::from("node_name"), Name::from("socket_name")));
/// ```
macro_rules! sref {
    (graph $field:expr) => {
        $crate::shader::graph::SocketRef::Graph($crate::shader::graph::Name::from($field))
    };

    // TODO: Remove this version in favor of the expr one
    (node $node:literal $field:literal) => {
        $crate::shader::graph::SocketRef::Node(
            $crate::shader::graph::NodeId::from($node),
            $crate::shader::graph::Name::from($field),
        )
    };

    (node $node:expr => $field:expr) => {
        $crate::shader::graph::SocketRef::Node(
            $crate::shader::graph::NodeId::from($node),
            $crate::shader::graph::Name::from($field),
        )
    };
}

#[macro_export]
/// Shorthand to reference sockets from the [Graph]s or other [Node]s, wrapped in an
/// [Option::Some]. Calls [sref] internally so the syntax is the same.
/// # Example
/// ```
/// use eray::{sref, ssref};
///
/// assert_eq!(ssref!(graph "value"), Some(sref!(graph "value")));
/// ```
macro_rules! ssref {
    ($($tree:tt)+) => {
        Some($crate::shader::graph::sref!($($tree)+))
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

#[derive(Debug, PartialEq, thiserror::Error)]
/// [Graph] error
pub enum Error {
    #[error("Graph output `{}` left unlinked", .0.to_string())]
    /// An unlinked and unset graph output is likely unintended.
    UnlinkeUnsetdGraphOutput(Name),

    #[error("Detected a cycle while validating the path {during:?}; cycle is from a `{}` socket to a `{}` socket, reaching node `{}`",
        source_socket.to_string(), target_socket.to_string(), detected.to_string())]
    /// Detected a cycle on the node with the given [NodeId].
    Cycle {
        /// Current path.
        during: Vec<NodeId>,
        /// Name of the socket the detected node was reached from.
        source_socket: Name,
        /// Name of the socket the current node was reached through.
        target_socket: Name,
        /// Node detected as already visited in the current path.
        detected: NodeId,
    },

    #[error("A shader function returned an error: {0}")]
    /// [Shader] returned with an error.
    Shader(super::shader::Error),

    #[error("Referencing missing {0:?} socket {}", .1.to_string())]
    /// Trying to get/set a non-existent socket.
    Missing(Side, Name),
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
/// use eray::{graph, node, ssref, shader::graph::{SocketValue, SocketType}};
///
/// graph! {
///     inputs:
///         "iFac": SocketValue::Value(Some(2.)),
///         "iName": SocketValue::Value(None),
///     nodes:
///         "identity": node! {
///             inputs:
///                 "value": (ssref!(graph "iFac"), SocketType::Value),
///             outputs:
///                 "value": SocketType::Value.into();
///             |_inputs, _outputs| Ok(())
///         },
///         "invert": node! {
///             inputs:
///                 "value": (ssref!(node "identity" "value"), SocketType::Value),
///             outputs:
///                 "value": SocketType::Value.into();
///         },
///     outputs:
///         "oFac": (ssref!(node "invert" "value"), SocketValue::Value(None)),
/// };
/// ```
macro_rules! graph {
    { $($field:ident $(: $($name:literal : $value:expr),+)? $(,)?),+ } => {
        $crate::shader::graph::Graph {
            $($field: [$($(($name.into(), $value)),+)?].into_iter().collect()),+,
            state: ::std::marker::PhantomData::<$crate::shader::graph::Unvalidated>,
        }
    };
}

impl Graph<Unvalidated> {
    /// Check the [unvalidated](Unvalidated) [Graph] for cycles.
    pub fn validate(self) -> Result<Graph<Validated>, Error> {
        let mut path: Vec<NodeId> = Vec::new();
        let mut visited: Vec<NodeId> = Vec::new();
        let mut next: VecDeque<NodeId> = VecDeque::new();

        // Graph outputs
        for (output, (socket_ref, value)) in self.outputs.iter() {
            // Check if graph output is connected to a socket or already has a value.
            let Some(socket_ref) = socket_ref else { 
                if value.is_none() {
                    return Err(Error::UnlinkeUnsetdGraphOutput(output.clone()))
                } else {
                    continue
                }
            };

            // Check that it is connected to a node.
            let SocketRef::Node(node_id, _socket) = socket_ref else {continue};

            // Ignore nodes connected to previously-handled graph outputs.
            if visited.contains(node_id) {
                continue;
            }

            next.push_back(node_id.clone());

            // Loop through nodes recursively (using the push_front trick).
            while let Some(current_node_id) = next.pop_front() {
                // Check that the current node exists.
                let Some(node) = self.nodes.get(&current_node_id) else {continue};

                visited.push(current_node_id.clone());
                path.push(current_node_id.clone());

                // Used to check if the recursion should end.
                let mut pushed_some = false;

                // Node inputs
                for (input, (socket_ref, _value)) in node.inputs() {
                    let Some(socket_ref) = socket_ref else {continue};
                    let SocketRef::Node(node_id, socket) = socket_ref else {continue};

                    // Check for cycles, i.e. if the node was already encountered in the path.
                    if path.contains(node_id) {
                        return Err(Error::Cycle {
                            detected: node_id.clone(),
                            target_socket: socket.clone(),
                            source_socket: input.clone(),
                            during: path,
                        });
                    }

                    // Ignore nodes visited from DFS starting from other graph outputs.
                    if visited.contains(node_id) {
                        continue;
                    }

                    next.push_front(node_id.clone());
                    pushed_some = true;
                }

                if !pushed_some {
                    path.pop();
                }
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
                            .unwrap_or_else(|| panic!("Output `{}` not found for node `{}`.", name.0, node_id.0)))
                        .clone();
                    }
                    SocketRef::Graph(name) => value = self.inputs.get(name).unwrap().clone(),
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
                .map(|(name, (_socket_ref, socket_type))| (name.clone(), *socket_type))
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

    /// Set a node input's socket reference.
    pub fn set_input(
        &mut self,
        name: &Name,
        socket_ref: Option<SocketRef>,
    ) -> Result<&mut Self, Error> {
        match self {
            Node::Graph(node) => node
                .inputs
                .get_mut(name)
                .ok_or_else(|| Error::Missing(Side::Input, name.clone()))
                .map(|(r#ref, _type)| *r#ref = socket_ref),
            Node::Imported(node) => node
                .inputs
                .get_mut(name)
                .ok_or_else(|| Error::Missing(Side::Input, name.clone()))
                .map(|(r#ref, _type)| *r#ref = socket_ref),
        }
        .map(|_| self)
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
            .map(|(name, (_socket_ref, socket_type))| (name.clone(), *socket_type))
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
/// Instantiate a node concisely.
///
/// # Examples
///
/// ```
/// use eray::{get_sv, ssref, node, shader::graph::{Node, Unvalidated, SocketValue, SocketType}};
/// let node: Node<Unvalidated> = node! {
///     inputs:
///         "value": (ssref!(graph "iFac"), SocketType::IValue.into()),
///     outputs:
///         "value": SocketValue::IValue(None);
///     |inputs, outputs| {
///         get_sv!( input | inputs  . "value" : Value > in_value);
///         get_sv!(output | outputs . "value" : Value > out_value);
///
///         *out_value.get_or_insert(0.) = in_value.unwrap_or(0.);
///
///         Ok(())
///     }
/// };
/// ```
/// The shader closure is optional and will be defaulted to a noop if empty.
///
/// ```
/// // With `imported` a HashMap<Name, ImportedNode<Unvalidated>>
/// use eray::{graph, node, ssref, shader::graph::{Name, ImportedNode, Unvalidated, SocketType, Graph}};
/// use std::collections::HashMap;
///
/// let mut imported = HashMap::<String, ImportedNode<Unvalidated>>::new();
/// imported.insert("node".to_string(), ImportedNode::from((
///     "node",
///     graph!{
///         inputs:
///             "value": SocketType::Value.into(),
///         nodes,
///         outputs,
///     }
/// )));
///
/// node!{
///     import "node" from imported,
///     inputs:
///         "value": (None, SocketType::Value),
/// };
/// ```
///
/// ```
/// use eray::{graph, node, ssref, shader::graph::{Graph, Unvalidated, SocketType}};
///
/// let sub_graph = graph!{
///     inputs:
///         "value": SocketType::Value.into(),
///     nodes,
///     outputs,
/// };
///
/// node!{
///     import graph "name" sub_graph,
///     inputs:
///         "value": (ssref!(graph "value"), SocketType::Value),
/// };
/// ```
macro_rules! node {
    ( import $name:literal from $imported:expr $(,)?) => {
        $crate::shader::graph::Node::Imported(
            $imported
                .get($name)
                .expect(format!("Could not find imported node `{}`. Imported nodes are: {}",
                    $name, $imported.keys().cloned().collect::<Vec<String>>().join(", ")).as_str()).clone()
        )
    };

    ( import $name:literal from $imported:expr $(, inputs: $($input:literal : $socket_ref:expr)+)? $(,)?) => {
        $crate::shader::graph::Node::Imported({
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

    ( import graph $name:literal $graph:expr $(, inputs: $($input:literal : $socket_ref:expr)+)? $(,)? ) => {
        $crate::shader::graph::Node::Imported({
            let mut res = $crate::shader::graph::ImportedNode::from(($name.to_string(), $graph));

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
        $crate::shader::graph::Node::Graph($crate::shader::graph::GraphNode {
            $($field: [$($(($o_name.into(), $value)),+)?].into_iter().collect()),+,
            shader: $crate::shader::shader::Shader::new($shader),
        })
    };

    { $($field:ident $(: $($o_name:literal : $value:expr),+)?),+ $(,)? $(;)? } => {
        $crate::shader::graph::Node::Graph($crate::shader::graph::GraphNode {
            $($field: [$($(($o_name.into(), $value)),+)?].into_iter().collect()),+,
            shader: $crate::shader::shader::Shader::default(),
        })
    };
}

// Export macros
pub use {graph, node, sref, ssref};

#[cfg(test)]
mod test {
    use super::*;
    use crate::{graph, node, ssref, get_sv};

    fn setup_imports() -> HashMap<String, ImportedNode<Unvalidated>> {
        std::iter::once((
            "identity".to_owned(),
            ImportedNode::from((
                "identity",
                graph! {
                    inputs:
                        "value": SocketValue::Value(Some(0.)),
                    nodes:
                        "id": node! {
                            inputs:
                                "value": (None, SocketType::Value),
                            outputs:
                                "value": SocketType::Value.into();
                            |inputs, outputs| {
                                get_sv!(input | inputs . "value" : Value > in_value);
                                get_sv!(output | outputs . "value" : Value > out_value);

                                *out_value.get_or_insert(0.) = in_value.unwrap_or(0.);

                                Ok(())
                            }
                        },
                    outputs:
                        "value": (ssref!(node "id" "value"), SocketType::Value.into()),
                },
            )),
        ))
        .collect()
    }

    #[cfg(test)]
    mod cycle_detection {
        use super::*;

        #[test]
        fn no_cycle() {
            let imported = setup_imports();

            let validation_result = graph! {
                inputs:
                    "value": SocketType::IValue.into(),
                nodes:
                    "a": node! {
                        import "identity" from imported,
                        inputs:
                            "value": (ssref!(graph "value"), SocketType::IValue)
                },
                outputs:
                    "value": (ssref!(node "a" "value"), SocketType::IValue.into()),
            }
            .validate();

            assert!(
                validation_result.is_ok(),
                "Expected a success, got `{validation_result:?}`"
            );
        }

        #[test]
        fn cycle() {
            let imported = setup_imports();

            let validation_result = graph! {
                inputs,
                nodes:
                    "a": node! {
                        import "identity" from imported,
                        inputs:
                            "value": (ssref!(node "b" "value"), SocketType::IValue),
                    },
                    "b": node! {
                        import "identity" from imported,
                        inputs:
                            "value": (ssref!(node "a" "value"), SocketType::IValue),
                    },
                outputs:
                    "value": (ssref!(node "a" "value"), SocketType::IValue.into()),
            }
            .validate();

            let expected = Error::Cycle {
                detected: NodeId("a".to_owned()),
                target_socket: "value".into(),
                source_socket: "value".into(),
                during: vec![NodeId("a".to_owned()), NodeId("b".to_owned())],
            };

            assert!(
                validation_result.is_err(),
                "Expected an error, got `{validation_result:?}`"
            );

            assert_eq!(validation_result.unwrap_err(), expected);
        }
    }

    #[test]
    fn macro_validity() {
        let manual = Graph {
            inputs: [
                (Name::from("iFac"), SocketValue::Value(Some(2.))),
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
                                SocketType::Value,
                            ),
                        ))
                        .collect(),
                        outputs: std::iter::once((Name::from("value"), SocketValue::Value(None)))
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
                                SocketType::Value,
                            ),
                        ))
                        .collect(),
                        outputs: [(Name::from("value"), SocketValue::Value(None))]
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
                    SocketValue::Value(None),
                ),
            ))
            .collect(),
            state: PhantomData::<Unvalidated>,
        };

        let r#macro = graph! {
            inputs:
                "iFac": SocketValue::Value(Some(2.)),
            nodes:
                "identity": node! {
                    inputs:
                        "value": (ssref!(graph "iFac"), SocketType::Value),
                    outputs:
                        "value": SocketType::Value.into()
                },
                "invert": node! {
                    inputs:
                        "value": (ssref!(node "identity" "value"), SocketType::Value),
                    outputs:
                        "value": SocketType::Value.into();
                },
            outputs:
                "oFac": (ssref!(node "invert" "value"), SocketValue::Value(None)),
        };

        assert_eq!(manual, r#macro);
    }
}
