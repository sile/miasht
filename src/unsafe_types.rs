use httparse;

use status::RawStatus;

pub type UnsafeHeader = httparse::Header<'static>;
pub type UnsafeRawStatus = RawStatus<'static>;
