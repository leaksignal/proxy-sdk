use std::borrow::Cow;

/// A reference to an upstream cluster. In envoy this is an encoded protobuf. See [`Upstream::envoy_upstream`].
#[derive(Default, Debug, Clone)]
pub struct Upstream<'a>(pub Cow<'a, [u8]>);

impl<'a> Upstream<'a> {
    pub const EMPTY: Upstream<'static> = Upstream(Cow::Borrowed(&[]));
}

impl<'a> From<String> for Upstream<'a> {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value.into_bytes()))
    }
}

impl<'a> From<Vec<u8>> for Upstream<'a> {
    fn from(value: Vec<u8>) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a, T: AsRef<[u8]>> From<&'a T> for Upstream<'a> {
    fn from(value: &'a T) -> Self {
        Self(Cow::Borrowed(value.as_ref()))
    }
}
