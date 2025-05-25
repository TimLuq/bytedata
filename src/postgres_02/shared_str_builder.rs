use alloc::boxed::Box;
use core::error::Error;

use bytes_1::BytesMut;
use postgres_types_02::{FromSql, IsNull, ToSql, Type};

use crate::SharedStrBuilder;

impl<'a> FromSql<'a> for SharedStrBuilder {
    #[inline]
    fn from_sql(
        ty: &Type,
        raw: &'a [u8],
    ) -> Result<SharedStrBuilder, Box<dyn Error + Sync + Send>> {
        <&str as FromSql>::from_sql(ty, raw).map(SharedStrBuilder::from)
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}

impl ToSql for SharedStrBuilder {
    #[inline]
    fn to_sql(&self, ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <&str as ToSql>::to_sql(&self.as_str(), ty, w)
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        <&str as ToSql>::accepts(ty)
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
