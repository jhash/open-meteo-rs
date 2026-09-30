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
        $(#[doc = concat!("`", $str, "`")] $variant),+
    }

    impl std::fmt::Display for $enum_name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.as_str())
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

    impl std::str::FromStr for $enum_name {
      type Err = crate::ConversionError;
      fn from_str(value: &str) -> Result<Self, Self::Err> {
          Self::try_from(value)
      }
    }

    impl $enum_name {
        /// All API strings for this parameter set (docs/tests).
        pub const ALL: &'static [&'static str] = &[$($str),+];

        /// Every variant, in the order of [`Self::ALL`].
        pub const VARIANTS: &'static [Self] = &[$(Self::$variant),+];

        /// The API string of this parameter.
        #[must_use]
        pub const fn as_str(&self) -> &'static str {
            match self {
                $(Self::$variant => $str),+
            }
        }
    }

    impl std::borrow::Borrow<str> for $enum_name {
      fn borrow(&self) -> &str {
          self.as_str()
      }
    }

    impl AsRef<str> for $enum_name {
      fn as_ref(&self) -> &str {
          self.as_str()
      }
    }
  };
}
