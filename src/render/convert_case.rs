use paste::paste;

macro_rules! define_is_sep {
    [ $( $sep:expr ),* ] => {
        macro_rules! is_sep {
            ( $ch:expr ) => {
                $( $ch == $sep )||*
            };
        }
    };
}

define_is_sep!['_', '-'];

macro_rules! define_joinedcase {
    ( $name:ident, $sep:expr, $upper:expr ) => {
        paste! {
            pub fn [<to_ $name>](s: &str) -> String {
                let mut res = String::new();
                let mut add_sep = false;
                for c in s.chars() {
                    if is_sep!(c) {
                        if !res.is_empty() {
                            add_sep = true;
                        }
                        continue;
                    }
                    if (c.is_uppercase() && !res.is_empty()) || add_sep {
                        res.push($sep);
                        add_sep = false;
                    }
                    if $upper {
                        res.push(c.to_ascii_uppercase());
                    } else {
                        res.push(c.to_ascii_lowercase());
                    }
                }
                res
            }
        }
    };

    ( $name:ident, $sep:expr ) => {
        define_joinedcase!($name, $sep, false);
    };
}

define_joinedcase!(snakecase, '_');

define_joinedcase!(kebabcase, '-');

define_joinedcase!(uppercase, '_', true);

pub fn to_pascalcase(s: &str) -> String {
    let mut res = String::new();
    let mut to_upper = true;
    for c in s.chars() {
        if is_sep!(c) {
            to_upper = true;
        } else if to_upper {
            res.push(c.to_ascii_uppercase());
            to_upper = false;
        } else {
            res.push(c);
        }
    }
    res
}

pub fn to_camelcase(s: &str) -> String {
    let mut res = String::new();
    let mut to_upper = false;
    let mut first = true;
    for c in s.chars() {
        if is_sep!(c) {
            to_upper = !first;
        } else if first {
            res.push(c.to_ascii_lowercase());
            first = false;
        } else if to_upper {
            res.push(c.to_ascii_uppercase());
            to_upper = false;
        } else {
            res.push(c);
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! define_case_test {
        ( $name:ident, $case:ident, $input:expr, $expected:expr ) => {
            paste! {
                #[test]
                fn [<$name _to_ $case _test>]() {
                    let result = [<to_ $case>]($input);
                    assert_eq!(result, $expected);
                }
            }
        };
    }

    macro_rules! case_tests {
        ( ($d:tt) [ $( $case:ident ),* ] ) => {
            $(
                paste! {
                    macro_rules! [<$case _tests>] {
                        { $d( $name:ident; $input:expr => $expected:expr ),* } => {
                            $d( define_case_test!($name, $case, $input, $expected); )*
                        };
                    }
                }
            )*
        };

        [ $( $case:ident ),* ] => {
            case_tests!(($) [ $($case),* ]);
        };
    }

    macro_rules! super_case_tests {
        {
            $( $case:ident => {
                $( $name:ident; $input:expr => $expected:expr ),*
            } )*
        } => {
            case_tests![ $( $case ),* ];
            $(
                paste! {
                    [<$case _tests>]! { $( $name; $input => $expected ),* }
                }
            )*
        };
    }

    super_case_tests! {
        snakecase => {
            snakecase; "hello_world" => "hello_world",
            pascalcase; "HelloWorld" => "hello_world",
            kebabcase; "hello-world" => "hello_world",
            camelcase; "helloWorld" => "hello_world",
            multicase; "Foo-barBaz" => "foo_bar_baz",
            underscores; "__Hello___World__" => "hello_world"
        }
        kebabcase => {
            snakecase; "hello_world" => "hello-world",
            pascalcase; "HelloWorld" => "hello-world",
            kebabcase; "hello-world" => "hello-world",
            camelcase; "helloWorld" => "hello-world",
            multicase; "Foo-barBaz" => "foo-bar-baz",
            underscores; "__Hello___World__" => "hello-world"
        }
        uppercase => {
            snakecase; "hello_world" => "HELLO_WORLD",
            pascalcase; "HelloWorld" => "HELLO_WORLD",
            kebabcase; "hello-world" => "HELLO_WORLD",
            camelcase; "helloWorld" => "HELLO_WORLD",
            multicase; "Foo-barBaz" => "FOO_BAR_BAZ",
            underscores; "__Hello___World__" => "HELLO_WORLD"
        }
        pascalcase => {
            snakecase; "hello_world" => "HelloWorld",
            pascalcase; "HelloWorld" => "HelloWorld",
            kebabcase; "hello-world" => "HelloWorld",
            camelcase; "helloWorld" => "HelloWorld",
            multicase; "Foo-barBaz" => "FooBarBaz",
            underscores; "__Hello___World__" => "HelloWorld"
        }
        camelcase => {
            snakecase; "hello_world" => "helloWorld",
            pascalcase; "HelloWorld" => "helloWorld",
            kebabcase; "hello-world" => "helloWorld",
            camelcase; "helloWorld" => "helloWorld",
            multicase; "Foo-barBaz" => "fooBarBaz",
            underscores; "__Hello___World__" => "helloWorld"
        }
    }
}
