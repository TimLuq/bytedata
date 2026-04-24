use alloc::boxed::Box;
use core::error::Error;

use bytes_1::BytesMut;
use postgres_types_02::{FromSql, IsNull, ToSql, Type};

use crate::SharedBytesBuilder;

impl<'a> FromSql<'a> for SharedBytesBuilder {
    #[inline]
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<SharedBytesBuilder, Box<dyn Error + Sync + Send>> {
        Ok(SharedBytesBuilder::from(raw))
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        matches!(ty, &Type::BYTEA)
    }
}

impl ToSql for SharedBytesBuilder {
    #[inline]
    fn to_sql(&self, _ty: &Type, w: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        w.extend_from_slice(self.as_slice());
        Ok(IsNull::No)
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        matches!(ty, &Type::BYTEA)
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        if !<Self as ToSql>::accepts(ty) {
            return Err(Box::new(postgres_types_02::WrongType::new::<Self>(
                ty.clone(),
            )));
        }
        self.to_sql(ty, out)
    }
}
