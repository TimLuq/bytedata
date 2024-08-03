impl crate::external::ExternalBytes for ::bytes_1::Bytes {
    const OPS: crate::external::ExternalOps<Self> =
        crate::external::ExternalOps::new(::bytes_1::Bytes::as_ref);
}
