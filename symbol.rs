/*!
 * Contains the global symbol string interner. The intention is that Symbol can be treated
 * as if it was a string, without actually having every symbol be a String allocation.
 *
 * TODO: Possibly switch this module to using *const pointers to never-freed data, rather
 * than the identifying usize "tokens" which are used now. This would make prevent ever
 * having to aquire a lock to Display symbols.
 */
use std::fmt;

use string_interner::StringInterner;
use std::sync::Mutex;

lazy_static! {
    static ref SYMBOL_INTERNER: Mutex<StringInterner<usize>> =
        Mutex::new(StringInterner::new());
}

fn acquire_lock() -> ::std::sync::MutexGuard<'static, StringInterner<usize>> {
    // FIXME: This blatantly assumes that the only time STRING_INTERNER will
    // be used by multiple threads is when running tests. This is completely
    // wrong if almost ANY KIND of multithreading is implemented later.
    let lock = {
        #[cfg(not(test))] {
            // Mutex::try_lock does NOT block.
            // When not running tests, we assume the program will always run
            // single-threaded, and should therefore getting the lock should
            // never block (assuming Symbol creation/to_string is correct
            // and doesn't leak a lock).
            SYMBOL_INTERNER.try_lock().expect("Failed to aquire lock on SYMBOL_INTERNER")
        }
        #[cfg(test)] {
            // Mutex::lock DOES block.
            // When running tests, many `#[test] fn ...`'s are run in parallel, sharing
            // the global STRING_INTERNER. We expect therefore to sometimes have to wait
            // to acquire the lock.
            SYMBOL_INTERNER.lock().expect("SYMBOL_INTERNER Mutex was poisoned!")
        }
    };
    lock
}

// By using `usize` here, we gurantee that we can later change this to be a pointer
// instead without changing the sizes of a lot of Expr types. This is good for FFI/ABI
// compatibility if I decide to change the way Symbol works.
#[derive(Copy, Clone, Eq, Hash)]
pub struct Symbol(usize);

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO?: Drop this lock before the write!. write! will block, and in pathological
        // cases, could cause other threads attempting to create/format symbols to block
        // longer than necessary in acquire_lock() (assuming the try_lock().expect(...) is
        // replaced with lock()).
        //
        // Average case performance might be better if, after acquiring the lock and
        // resolve to a &str, we create a String and immediately drop the lock, *then* do
        // the write!. That prevents holding the lock while doing a write!.
        let lock = acquire_lock();
        // TODO: Replace this with resolve_unchecked once I'm confident that's correct.
        let s: &str = lock.resolve(self.0)
            .expect("Failed to resolve Symbol from global SYMBOL_INTERNER");
        write!(f, "{}", s)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lock = acquire_lock();
        // TODO: Replace this with resolve_unchecked once I'm confident that's correct.
        let s: &str = lock.resolve(self.0)
            .expect("Failed to resolve Symbol from global SYMBOL_INTERNER");
        write!(f, "Symbol({}: {})", self.0, s)
    }
}

impl<T: Into<String> + AsRef<str>> From<T> for Symbol {
    fn from(s: T) -> Symbol {
        let mut lock = acquire_lock();
        let sym: usize = lock.get_or_intern(s);
        Symbol(sym)
    }
}

impl PartialEq<str> for Symbol {
    fn eq(&self, other: &str) -> bool {
        let mut lock = acquire_lock();
        let other_sym: usize = lock.get_or_intern(other);
        self.0 == other_sym
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Symbol) -> bool {
        self.0 == other.0
    }
}
