use alloc::boxed::Box;
use core::error::Error;

use bytes_1::BytesMut;
use postgres_types_02::{FromSql, IsNull, ToSql, Type};

use crate::SharedBytesBuilder;

impl<'a> FromSql<'a> for SharedBytesBuilder {
    #[inline]
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<SharedBytesBuilder, Box<dyn Error + Sync + Send>> {
        <&[u8] as FromSql>::from_sql(ty, raw).map(SharedBytesBuilder::from)
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        <&[u8] as FromSql>::accepts(ty)
    }
}

impl ToSql for SharedBytesBuilder {
    #[inline]
    fn to_sql(&self, ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <&[u8] as ToSql>::to_sql(&self.as_slice(), ty, w)
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        <&[u8] as ToSql>::accepts(ty)
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        if !<&str as ToSql>::accepts(ty) {
            return Err(Box::new(postgres_types_02::WrongType::new::<Self>(
                ty.clone(),
            )));
        }
        self.to_sql(ty, out)
    }
}
