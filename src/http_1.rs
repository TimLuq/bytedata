impl TryFrom<crate::ByteData<'_>> for http_1::HeaderValue {
    type Error = http_1::header::InvalidHeaderValue;

    #[allow(clippy::missing_inline_in_public_items)]
    fn try_from(value: crate::ByteData<'_>) -> Result<Self, Self::Error> {
        Self::from_maybe_shared(bytes_1::Bytes::from(value))
    }
}
