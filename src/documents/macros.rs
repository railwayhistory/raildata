
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
            fn construct<C>(value: $crate::load::yaml::Value,
                            context: &mut C)
                            -> Result<Self,
                                      $crate::load::construct::Failed>
                         where C: $crate::load::construct::Context {
                $crate::documents::types::Marked::construct(value, context)
                       .map(|res: $crate::documents::types::Marked<$name>|
                                    res.into_value())
            }
        }

        impl $crate::load::construct::Constructable
                         for $crate::documents::types::Marked<$name> {
            fn construct<C>(value: $crate::load::yaml::Value,
                            context: &mut C)
                            -> Result<Self,
                                      $crate::load::construct::Failed>
                         where C: $crate::load::construct ::Context {
                let text = value.into_string(context)?;
                let res = text.try_map(|plain| match plain.as_ref() {
                    $(
                        $yaml => Ok($name::$variant),
                    )*
                    _ => Err($crate::documents::types::EnumError::new(plain))
                });
                res.map_err(|err| {
                    context.push_error(err);
                    $crate::load::construct::Failed
                })
            }
        }
    };
}

