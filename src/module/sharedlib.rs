use std::{path::Path, sync::Arc};

use libloading::Library;

use crate::error::Error;

// A symbol from a shared library.
pub trait Symbol<T> {
    /// Provides access to the data that this symbol references.
    ///
    /// # Unsafety
    /// If the data that this symbol references contains pointers to other things in the shared
    /// library, and `T: Clone`, we can obtain a clone of the data and use it to outlast the
    /// library. To prevent this, the return of this function should never be cloned.
    unsafe fn get(&self) -> T;
}

/// A pointer to a shared function which provides no protection against outliving its library.
pub type FuncUnsafe<T> = T;

/// A pointer to a shared function which allows a user-provided ref-counting implementation to avoid outliving its library.
#[derive(Debug)]
pub struct FuncTracked<T, TLib> {
    func: FuncUnsafe<T>,
    lib: TLib,
}

/// A pointer to a shared function which uses atomic ref-counting to avoid outliving its library.
pub type FuncArc<T> = FuncTracked<T, Arc<LibUnsafe>>;

/// A shared library which does not track its [`Symbols`].
/// The inner library may be dropped at any time, even if it has loose symbols.
pub type LibUnsafe = Library;

/// A shared library which implements [LibTracked](struct.LibTracked.html) with atomic ref-counting to track its [Symbols](trait.Symbol.html).
pub type LibArc = LibTracked<Arc<LibUnsafe>>;

/// A shared library which which allows a user-provided ref-counting implementation to track its [`Symbols`].
/// The inner library will not be droped until all of the ref-counts are dropped.
#[derive(Clone, Debug)]
pub struct LibTracked<TLib> {
    inner: TLib,
}

impl<TLib> LibTracked<TLib>
where
    TLib: AsRef<LibUnsafe> + Clone + From<LibUnsafe>,
{
    pub unsafe fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let lib = LibUnsafe::new(path.as_ref())?;
        let inner = TLib::from(lib);
        Ok(LibTracked { inner: inner })
    }

    /// Finds and returns a function symbol of the shared library.
    pub unsafe fn find_func<T: Copy, TStr: AsRef<str>>(
        &self,
        symbol: TStr,
    ) -> Result<FuncTracked<T, TLib>, Error> {
        let lib = self.inner.as_ref();
        let func = lib.get::<T>(symbol.as_ref().as_bytes())?;
        let func = std::mem::transmute_copy(&func);
        Ok(FuncTracked::new(func, self.inner.clone()))
    }
}

impl<T, TLib> FuncTracked<T, TLib> {
    /// Creates a new [FuncTracked](struct.FuncTracked.html).
    /// This should only be called within the library.
    fn new(func: FuncUnsafe<T>, lib: TLib) -> Self {
        FuncTracked {
            func: func,
            lib: lib,
        }
    }
}

impl<T: Copy> Symbol<T> for FuncUnsafe<T> {
    unsafe fn get(&self) -> T {
        self.clone()
    }
}

impl <T: Copy, TLib> Symbol<T> for FuncTracked<T, TLib> {
    unsafe fn get(&self) -> T {
        self.func
    }
}

impl<T: Copy, TLib: Clone> Clone for FuncTracked<T, TLib> {
    fn clone(&self) -> Self {
        FuncTracked {
            func: self.func,
            lib: self.lib.clone(),
        }
    }
}
