//! Shader implementation, a [Shader] being a wrapper containing functions of a specific type
//! signature with convenience functions.

use std::collections::HashMap;

use super::graph::{Name, SocketType, SocketValue};

#[derive(Debug, PartialEq)]
/// Possible errors returned during a [Shader]'s lifecycle.
pub enum Error {
    /// Missing socket.
    Missing(Side, Name),
    /// Wrong type for socket.
    MismatchedTypes((Name, SocketType), (Name, SocketType)),
    /// Tried to unwrap a socket with the wrong expected [SocketType].
    InvalidType {
        /// [Name] of the socket.
        name: Name,
        /// Requested [SocketType].
        got: SocketType,
        /// Actual [SocketType] of the socket.
        expected: SocketType,
    },
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
    /// Shader::new(|inputs, outputs| {
    ///     get_sv!(input  | inputs  . "value" : Number > in_value);
    ///     get_sv!(output | outputs . "value" : Number > out_value);
    ///
    ///     *out_value.get_or_insert(0.) = -in_value.unwrap_or(0.);
    ///
    ///     Ok(())
    /// })
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
    Fn(&HashMap<Name, SocketValue>, &mut HashMap<Name, SocketValue>) -> Result<(), Error>
{
    /// Clone the boxed function.
    fn clone_box(&self) -> Box<dyn CloneFn>;
}

impl<F: Clone + 'static> CloneFn for F
where
    F: Fn(&HashMap<Name, SocketValue>, &mut HashMap<Name, SocketValue>) -> Result<(), Error>,
{
    fn clone_box(&self) -> Box<dyn CloneFn> {
        Box::new(self.clone())
    }
}

#[macro_export]
/// [get](std::collections::HashMap::get)s the desired input/output field with error reporting
macro_rules! get_sv {
    (input | $hashmap:ident . $field:literal : $type:ident > $name:ident) => {
        let $name = $hashmap.get(&$field.into()).ok_or_else(|| {
            crate::shader::shader::Error::Missing(crate::shader::shader::Side::Input, $field.into())
        })?;

        #[rustfmt::skip]
        let crate::shader::graph::SocketValue::$type($name) = $name
            else {
                return Err(crate::shader::shader::Error::InvalidType {
                    name: $field.into(),
                    got: crate::shader::graph::SocketType::from($name),
                    expected: crate::shader::graph::SocketType::$type,
                });
            };
    };

    (output | $hashmap:ident . $field:literal : $type:ident > $name:ident) => {
        let $name = $hashmap.get_mut(&$field.into()).ok_or_else(|| {
            crate::shader::shader::Error::Missing(
                crate::shader::shader::Side::Output,
                $field.into(),
            )
        })?;

        #[rustfmt::skip]
        let crate::shader::graph::SocketValue::$type(ref mut $name) = $name
            else {
                return Err(crate::shader::shader::Error::InvalidType {
                    name: $field.into(),
                    got: crate::shader::graph::SocketType::from($name),
                    expected: crate::shader::graph::SocketType::$type,
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
        inputs.insert("value".into(), SocketType::Number.into());

        Shader::new(|inputs, outputs| {
            get_sv!( input | inputs  . "value" : Number > in_value);
            get_sv!(output | outputs . "value" : Number > out_value);

            let initial = out_value.clone();

            *out_value.get_or_insert(0.) += in_value.unwrap_or(0.);

            let modified = out_value.clone();

            assert_ne!(initial, modified);

            Ok(())
        })
        .call(&inputs, &mut inputs.clone())
        .unwrap();
    }
}
