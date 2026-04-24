use alloc::boxed::Box;
use core::error::Error;

use bytes_1::BytesMut;
use postgres_types_02::{FromSql, IsNull, ToSql, Type};

use crate::ByteData;

impl<'a> FromSql<'a> for ByteData<'a> {
    #[inline]
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<ByteData<'a>, Box<dyn Error + Sync + Send>> {
        Ok(ByteData::from_borrowed(raw))
    }

    #[inline]
    fn accepts(ty: &Type) -> bool {
        matches!(ty, &Type::BYTEA)
    }
}

impl ToSql for ByteData<'_> {
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

#[test]
fn test_postgres_02_bytedata() {
    let test_value = crate::ByteData::from_static(b"Hello, world!");
    ToSql::to_sql_checked(
        &test_value,
        &Type::BYTEA,
        &mut postgres_types_02::private::BytesMut::new(),
    )
    .unwrap();
}
