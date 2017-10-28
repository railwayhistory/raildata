
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

        impl $crate::load::construct::Constructable for $name {
            fn construct(value: $crate::load::yaml::Value,
                         context: &mut $crate::load::construct
                                             ::ConstructContext)
                            -> Result<Self,
                                      $crate::load::construct::Failed> {
                $crate::types::Marked::construct(value, context)
                       .map(|res: $crate::types::Marked<$name>|
                                    res.into_value())
            }
        }

        impl $crate::load::construct::Constructable
                         for $crate::types::Marked<$name> {
            fn construct(value: $crate::load::yaml::Value,
                         context: &mut $crate::load::construct
                                             ::ConstructContext)
                            -> Result<Self,
                                      $crate::load::construct::Failed> {
                let text = value.into_string(context)?;
                let res = text.try_map(|plain| match plain.as_ref() {
                    $(
                        $yaml => Ok($name::$variant),
                    )*
                    _ => Err($crate::types::marked::EnumError::new(plain))
                });
                res.map_err(|err| {
                    context.push_error(err);
                    $crate::load::construct::Failed
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

