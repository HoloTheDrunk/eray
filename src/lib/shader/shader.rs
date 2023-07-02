//! Shader implementation, a [Shader] being a wrapper containing functions of a specific type
//! signature with convenience functions.

use super::graph::{Name, SocketType, SocketValue};

use std::collections::HashMap;

#[derive(Debug, PartialEq, thiserror::Error)]
/// Possible errors returned during a [Shader]'s lifecycle.
pub enum Error {
    #[error("Missing {} on {0:?} side", .1.to_string())]
    /// Missing socket or value.
    Missing(Side, Name),

    #[error("Missing {} on {0:?} side", .1.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(", "))]
    /// Missing many sockets or values.
    MissingMany(Side, Vec<Name>),

    #[error("Mismatched type between {} ({:?}) and {} ({:?})", 
        {let (name, _) = .0; name.to_string()}, {let (_, ty) = .0; ty},
        {let (name, _) = .1; name.to_string()}, {let (_, ty) = .1; ty})]
    /// Wrong type for socket.
    MismatchedTypes((Name, SocketType), (Name, SocketType)),

    #[error("Invalid type {got:?} for {}, expected {expected:?}", .name.to_string())]
    /// Tried to unwrap a socket with the wrong expected [SocketType].
    InvalidType {
        /// [Name] of the socket.
        name: Name,
        /// Requested [SocketType].
        got: SocketType,
        /// Actual [SocketType] of the socket.
        expected: SocketType,
    },

    #[error("Unknown error{}", .0.as_ref().map_or("".to_string(), |v| format!(": {v}")))]
    /// Unknown or untyped error
    Unknown(Option<String>),
}

#[derive(Clone, Debug, PartialEq)]
/// Socket side.
pub enum Side {
    #[allow(missing_docs)]
    Input,
    #[allow(missing_docs)]
    Output,
}

/// Shader container
pub struct Shader {
    func: Box<dyn CloneFn>,
}

impl Shader {
    /// Creates a [Shader] from a shader function.
    /// # Example
    /// ```
    /// use eray::{get_sv, shader::{shader::Shader}};
    ///
    /// Shader::new(|inputs, outputs| {
    ///     get_sv!(input  | inputs  . "value" : Value > in_value);
    ///     get_sv!(output | outputs . "value" : Value > out_value);
    ///
    ///     *out_value.get_or_insert(0.) = -in_value.unwrap_or(0.);
    ///
    ///     Ok(())
    /// });
    /// ```
    pub fn new(
        func: fn(&HashMap<Name, SocketValue>, &mut HashMap<Name, SocketValue>) -> Result<(), Error>,
    ) -> Self {
        Self {
            func: Box::new(func),
        }
    }

    /// Execute the contained function.
    pub fn call(
        &self,
        inputs: &HashMap<Name, SocketValue>,
        outputs: &mut HashMap<Name, SocketValue>,
    ) -> Result<(), Error> {
        (self.func)(inputs, outputs)
    }
}

impl Default for Shader {
    fn default() -> Self {
        Self::new(|_inputs, _outputs| Ok(()))
    }
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        Self {
            func: self.func.clone_box(),
        }
    }
}

/// Intermediary trait to allow boxing function inside a struct.
pub trait CloneFn:
    Send + Sync + Fn(&HashMap<Name, SocketValue>, &mut HashMap<Name, SocketValue>) -> Result<(), Error>
{
    /// Clone the boxed function.
    fn clone_box(&self) -> Box<dyn CloneFn>;
}

impl<F: Clone + 'static> CloneFn for F
where
    F: Send
        + Sync
        + Fn(&HashMap<Name, SocketValue>, &mut HashMap<Name, SocketValue>) -> Result<(), Error>,
{
    fn clone_box(&self) -> Box<dyn CloneFn> {
        Box::new(self.clone())
    }
}

#[macro_export]
/// [get](std::collections::HashMap::get)s the desired input/output field with error reporting
///
/// # Example
///
/// ```
/// use eray::{get_sv, shader::graph::{Name, SocketType, SocketValue}};
/// use std::collections::HashMap;
///
/// let mut inputs = HashMap::<Name, SocketValue>::new();
/// inputs.insert("value".into(), SocketType::Value.into());
/// let mut outputs = inputs.clone();
///
/// (|| {
///     get_sv!( input | inputs  . "value" : Value > in_value);
///     get_sv!(output | outputs . "value" : Value > out_value);
///     Ok(())
/// })().unwrap();
/// ```
macro_rules! get_sv {
    (input | $hashmap:ident . $field:literal : $type:ident > $name:ident) => {
        let $name = $hashmap.get(&$field.into()).ok_or_else(|| {
            $crate::shader::shader::Error::Missing(
                $crate::shader::shader::Side::Input,
                $field.into(),
            )
        })?;

        #[rustfmt::skip]
        let $crate::shader::graph::SocketValue::$type($name) = $name
            else {
                return Err($crate::shader::shader::Error::InvalidType {
                    name: $field.into(),
                    got: $crate::shader::graph::SocketType::from($name),
                    expected: $crate::shader::graph::SocketType::$type,
                });
            };
    };

    (output | $hashmap:ident . $field:literal : $type:ident > $name:ident) => {
        let $name = $hashmap.get_mut(&$field.into()).ok_or_else(|| {
            $crate::shader::shader::Error::Missing(
                $crate::shader::shader::Side::Output,
                $field.into(),
            )
        })?;

        #[rustfmt::skip]
        let $crate::shader::graph::SocketValue::$type(ref mut $name) = $name
            else {
                return Err($crate::shader::shader::Error::InvalidType {
                    name: $field.into(),
                    got: $crate::shader::graph::SocketType::from($name),
                    expected: $crate::shader::graph::SocketType::$type,
                });
            };
    };
}

pub use get_sv;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn shader_function_type() {
        let mut inputs = HashMap::new();
        inputs.insert("value".into(), SocketType::Value.into());

        Shader::new(|inputs, outputs| {
            get_sv!( input | inputs  . "value" : Value > in_value);
            get_sv!(output | outputs . "value" : Value > out_value);

            let initial = *out_value;

            *out_value.get_or_insert(0.) += in_value.unwrap_or(0.);

            let modified = *out_value;

            assert_ne!(initial, modified);

            Ok(())
        })
        .call(&inputs, &mut inputs.clone())
        .unwrap();
    }
}
