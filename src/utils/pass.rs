use std::process::exit;

use crate::utils::error::MultiError;

#[macro_export]
macro_rules! run_passes {
    (($config:expr, $errors:ident)) => {
        $config
    };

    (
        ($config:expr, $errors:ident)
        => $first_pass:ident$(($($first_arg:expr),+$(,)?))?
        $(=> $pass:ident$(($($arg:expr),+$(,)?))? $(| $post:ident)?)*
    ) => {
        run_passes!(
            ($first_pass::do_pass($config, &mut $errors $($(, $first_arg)+)?), $errors)
            $(=> $pass$(($($arg),+))? $(| $post)?)*
        )
    };

    (
        ($config:expr, $errors:ident)
        => $first_pass:ident$(($($first_arg:expr),+$(,)?))? | $first_post:ident
        $(=> $pass:ident$(($($arg:expr),+$(,)?))? $(| $post:ident)?)*
    ) => {
        run_passes!(
            (crate::utils::pass::$first_post(
                $first_pass::do_pass($config, &mut $errors $($(, $first_arg)+)?),
                &$errors,
            ), $errors)
            $(=> $pass$(($($arg),+))? $(| $post)?)*
        )
    };
}

pub fn exit_on_error<T>(value: T, errors: &MultiError) -> T {
    for error in errors.iter() {
        println!("{}", error);
    }
    if let Some(error_kind) = errors.error_kind() {
        exit(error_kind as i32);
    }
    value
}
