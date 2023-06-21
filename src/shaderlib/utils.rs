#[macro_export]
macro_rules! missing_socket_error_vec {
    ($($name:ident),+ $(,)?) => {
        {
            let result: Vec<(bool, &str)> = vec![
                $(
                    ($name.is_none(), stringify!($name))
                ),+
            ];
            result
        }
    };
}

#[macro_export]
macro_rules! handle_missing_socket_values {
    ($($name:ident),+ $(,)?) => {
        let missing: Vec<crate::shader::graph::Name> = crate::shaderlib::utils::missing_socket_error_vec![$($name),+]
            .into_iter()
            .filter_map(|(is_none, name)| is_none.then(|| name.into()))
            .collect();

        if missing.len() != 0 {
            return Err(crate::shader::shader::Error::MissingMany(
                crate::shader::shader::Side::Input,
                missing
            ));
        }

        $(
            let $name = $name.as_ref().unwrap();
        )+
    };
}

pub use {handle_missing_socket_values, missing_socket_error_vec};
