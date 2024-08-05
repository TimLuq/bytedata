impl crate::external::ExternalBytes for ::bytes_1::Bytes {
    const OPS: crate::external::ExternalOps<Self> = crate::external::ExternalOps::new(Self::as_ref);
}
