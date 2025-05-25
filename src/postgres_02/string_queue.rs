use alloc::boxed::Box;
use core::error::Error;

use bytes_1::BytesMut;
use postgres_types_02::{FromSql, IsNull, ToSql, Type};

use crate::StringQueue;

impl<'a> FromSql<'a> for StringQueue<'a> {
    #[inline]
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<StringQueue<'a>, Box<dyn Error + Sync + Send>> {
        <&str as FromSql>::from_sql(ty, raw).map(StringQueue::from)
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        <&str as FromSql>::accepts(ty)
    }
}

impl ToSql for StringQueue<'_> {
    #[inline]
    fn to_sql(&self, _ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        for chunk in self.chunks() {
            w.extend_from_slice(chunk.as_bytes());
        }
        Ok(IsNull::No)
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
