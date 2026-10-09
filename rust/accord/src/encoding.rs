//! serde for the types whose wire form is their canonical text.

/// implements `Serialize` as the `Display` text and `Deserialize` through
/// `FromStr`, so a malformed text is rejected at the boundary instead of
/// producing a value that breaks the grammar.
macro_rules! text_serde {
    ($ty:ty) => {
        #[cfg(feature = "serde")]
        impl serde::Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.collect_str(self)
            }
        }

        #[cfg(feature = "serde")]
        impl<'de> serde::Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let text = <String as serde::Deserialize>::deserialize(d)?;
                text.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

pub(crate) use text_serde;
