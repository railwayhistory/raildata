
macro_rules! optional_enum {
    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $(
                $(#[$variant_attr:meta])*
                ( $variant:ident => $yaml:expr ),
            )*

            default $default:ident
        }
    ) => {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name {
            $( $(#[$variant_attr])* $variant ),*,
        }

        impl $name {
            pub fn from_yaml(item: Option<ValueItem>,
                             builder: &::collection::CollectionBuilder)
                             -> Result<Self, ()> {
                if let Some(item) = item {
                    let item = item.into_string_item(builder)?;
                    match item.as_ref().as_ref() {
                        $( $yaml => Ok($name::$variant), )*
                        _ => {
                            builder.error((item.source(),
                                           format!("invalid value '{}'",
                                                   item.value())));
                            Err(())
                        }
                    }
                }
                else {
                    Ok($name::$default)
                }
            }
        }

        impl ::load::yaml::FromYaml for $name {
            fn from_yaml(item: ValueItem,
                         builder: &::collection::CollectionBuilder)
                         -> Result<Self, ()> {
                let item = item.into_string_item(builder)?;
                match item.as_ref().as_ref() {
                    $( $yaml => Ok($name::$variant), )*
                    _ => {
                        builder.error((item.source(),
                                       format!("invalid value '{}'",
                                               item.value())));
                        Err(())
                    }
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> ::std::fmt::Result {
                match *self {
                    $( $name::$variant => f.write_str($yaml), )*
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name::$default
            }
        }
    }
}


macro_rules! mandatory_enum {
    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $(
                $(#[$variant_attr:meta])*
                ( $variant:ident => $yaml:expr ),
            )*
        }
    ) => {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum $name {
            $( $(#[$variant_attr])* $variant ),*,
        }

        impl ::load::yaml::FromYaml for $name {
            fn from_yaml(item: ValueItem,
                         builder: &::collection::CollectionBuilder)
                         -> Result<Self, ()> {
                let item = item.into_string_item(builder)?;
                match item.as_ref().as_ref() {
                    $( $yaml => Ok($name::$variant), )*
                    _ => {
                        builder.error((item.source(),
                                       format!("invalid value '{}'",
                                               item.value())));
                        Err(())
                    }
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter)
                   -> ::std::fmt::Result {
                match *self {
                    $( $name::$variant => f.write_str($yaml), )*
                }
            }
        }
    }
}
