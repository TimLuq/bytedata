use alloc::boxed::Box;
use core::error::Error;

use bytes_1::BytesMut;
use postgres_types_02::{FromSql, IsNull, ToSql, Type};

use crate::ByteQueue;

impl<'a> FromSql<'a> for ByteQueue<'a> {
    #[inline]
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<ByteQueue<'a>, Box<dyn Error + Sync + Send>> {
        <&[u8] as FromSql>::from_sql(ty, raw).map(ByteQueue::from)
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        <&[u8] as FromSql>::accepts(ty)
    }
}

impl ToSql for ByteQueue<'_> {
    #[inline]
    fn to_sql(&self, _ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        for chunk in self.chunks() {
            w.extend_from_slice(chunk.as_slice());
        }
        Ok(IsNull::No)
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
