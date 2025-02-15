// Tagged dispatch mechanism for resolving the behavior of `anyhow!($expr)`.
//
// When anyhow! is given a single expr argument to turn into anyhow::Error, we
// want the resulting Error to pick up the input's implementation of source()
// and backtrace() if it has a std::error::Error impl, otherwise require nothing
// more than Display and Debug.
//
// Expressed in terms of specialization, we want something like:
//
//     trait AnyhowNew {
//         fn new(self) -> Error;
//     }
//
//     impl<T> AnyhowNew for T
//     where
//         T: Display + Debug + Send + Sync + 'static,
//     {
//         default fn new(self) -> Error {
//             /* no std error impl */
//         }
//     }
//
//     impl<T> AnyhowNew for T
//     where
//         T: std::error::Error + Send + Sync + 'static,
//     {
//         fn new(self) -> Error {
//             /* use std error's source() and backtrace() */
//         }
//     }
//
// Since specialization is not stable yet, instead we rely on autoref behavior
// of method resolution to perform tagged dispatch. Here we have two traits
// AdhocKind and TraitKind that both have an anyhow_kind() method. AdhocKind is
// implemented whether or not the caller's type has a std error impl, while
// TraitKind is implemented only when a std error impl does exist. The ambiguity
// is resolved by AdhocKind requiring an extra autoref so that it has lower
// precedence.
//
// The anyhow! macro will set up the call in this form:
//
//     #[allow(unused_imports)]
//     use $crate::private::{AdhocKind, TraitKind};
//     let error = $msg;
//     (&error).anyhow_kind().new(error)

use crate::Error;
use std::error::Error as StdError;
use std::fmt::{Debug, Display};

#[cfg(backtrace)]
use std::backtrace::Backtrace;

pub struct Adhoc;

pub trait AdhocKind: Sized {
    #[inline]
    fn anyhow_kind(&self) -> Adhoc {
        Adhoc
    }
}

impl<T> AdhocKind for &T where T: ?Sized + Display + Debug + Send + Sync + 'static {}

impl Adhoc {
    pub fn new<M>(self, message: M) -> Error
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Error::from_adhoc(message, backtrace!())
    }
}

pub struct Trait;

pub trait TraitKind: Sized {
    #[inline]
    fn anyhow_kind(&self) -> Trait {
        Trait
    }
}

impl<T> TraitKind for T where T: StdError + Send + Sync + 'static {}

impl Trait {
    pub fn new<E>(self, error: E) -> Error
    where
        E: StdError + Send + Sync + 'static,
    {
        Error::from_std(error, backtrace!())
    }
}
