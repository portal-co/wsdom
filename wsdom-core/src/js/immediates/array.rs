use core::fmt::Write;

use crate::serialize::UseInJsCode;

impl<'a, U, const N: usize> UseInJsCode for [&'a U; N]
where
    U: UseInJsCode,
{
    fn serialize_to(&self, buf: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        buf.write_char('[')?;
        for item in self.iter() {
            item.serialize_to(buf)?;
            buf.write_char(',')?;
        }
        buf.write_char(']')?;
        Ok(())
    }
}
