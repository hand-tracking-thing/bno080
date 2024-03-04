/// empty
pub trait Format {}

/// no-op
macro_rules! println {
    ($($tt:tt)*) => {};
}

pub(crate) use println;
