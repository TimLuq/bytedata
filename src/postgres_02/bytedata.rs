use alloc::boxed::Box;
use core::error::Error;

use bytes_1::BytesMut;
use postgres_types_02::{FromSql, IsNull, ToSql, Type};

use crate::ByteData;

impl<'a> FromSql<'a> for ByteData<'a> {
    #[inline]
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<ByteData<'a>, Box<dyn Error + Sync + Send>> {
        <&[u8] as FromSql>::from_sql(ty, raw).map(ByteData::from_borrowed)
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        <&[u8] as FromSql>::accepts(ty)
    }
}

impl ToSql for ByteData<'_> {
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
