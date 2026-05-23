macro_rules! api_param_enum {
  (
    $(#[$meta:meta])*
    $enum_name:ident,
    invalid = $invalid_variant:ident,
    { $($variant:ident => $str:literal),+ $(,)? }
  ) => {
    $(#[$meta])*
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum $enum_name {
        $($variant),+
    }

    impl std::fmt::Display for $enum_name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(match self {
                $(Self::$variant => $str),+
            })
        }
    }

    impl From<$enum_name> for String {
        fn from(value: $enum_name) -> Self {
            value.to_string()
        }
    }

    impl TryFrom<&str> for $enum_name {
      type Error = crate::ConversionError;
      fn try_from(value: &str) -> Result<Self, Self::Error> {
          match value {
              $($str => Ok(Self::$variant),)+
              _ => Err(crate::ConversionError::$invalid_variant {
                  name: value.to_string(),
              }),
          }
      }
    }

    impl $enum_name {
        /// All API strings for this parameter set (docs/tests).
        pub const ALL: &'static [&'static str] = &[$($str),+];
    }

    impl std::borrow::Borrow<str> for $enum_name {
      fn borrow(&self) -> &str {
          match self {
              $(Self::$variant => $str),+
          }
      }
    }

    impl AsRef<str> for $enum_name {
      fn as_ref(&self) -> &str {
          match self {
              $(Self::$variant => $str),+
          }
      }
    }
  };
}
