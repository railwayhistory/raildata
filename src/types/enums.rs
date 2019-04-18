
macro_rules! data_enum {
    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $(
                $( #[$variant_attr:meta] )*
                {$variant:ident: $yaml:expr}
            )*

            default $default:ident
        }
    ) => {
        data_enum! {
            $(#[$attr])*
            pub enum $name {
                $(
                    $(#[$variant_attr])*
                    { $variant: $yaml }
                )*
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$default
            }
        }
    };

    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $(
                $( #[$variant_attr:meta] )*
                {$variant:ident: $yaml:expr}
            )*
        }
    ) => {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name {
            $( $(#[$variant_attr])* $variant ),*,
        }

        impl<C> $crate::load::yaml::FromYaml<C> for $name {
            fn from_yaml(
                value: $crate::load::yaml::Value,
                context: &C,
                report: &mut $crate::load::report::PathReporter
            ) -> Result<Self, $crate::load::report::Failed> {
                $crate::types::Marked::from_yaml(value, context, report)
                       .map(|res: $crate::types::Marked<$name>|
                                    res.into_value())
            }
        }

        impl<C> $crate::load::yaml::FromYaml<C>
        for $crate::types::Marked<$name> {
            fn from_yaml(
                value: $crate::load::yaml::Value,
                _: &C,
                report: &mut $crate::load::report::PathReporter
            ) -> Result<Self, $crate::load::report::Failed> {
                let text = value.into_string(report)?;
                let res = text.try_map(|plain| match plain.as_ref() {
                    $(
                        $yaml => Ok($name::$variant),
                    )*
                    _ => Err($crate::types::enums::EnumError::new(plain))
                });
                res.map_err(|err| {
                    report.error(err);
                    $crate::load::report::Failed
                })
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> ::std::fmt::Result {
                f.write_str(match *self {
                    $(
                        $name::$variant => $yaml,
                    )*
                })
            }
        }
    };
}


//------------ EnumError -----------------------------------------------------

#[derive(Clone, Debug, Display)]
#[display(fmt="invalid enum value '{}'", _0)]
pub struct EnumError(String);

impl EnumError {
    pub fn new(variant: String) -> Self {
        EnumError(variant)
    }
}

