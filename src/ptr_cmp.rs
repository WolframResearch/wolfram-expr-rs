use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::{Expr, ExprKind};


/// [`Expr`] wrapper that compares by reference instead of by value.
///
/// The standard [`Expr`] type uses *value* semantics for comparision: two [`Expr`]
/// instances are considered equal iff the Wolfram expression values they contain
/// are the same.
///
/// [`ExprRefCmp`] uses *reference* semantics for comparision: two [`ExprRefCmp`]
/// instances are considered equal iff they are pointers to the same
/// reference-counted expression allocation.
///
/// The [`Hash`] and [`PartialEq`] implementations for this type use the pointer
/// address of the reference counted expression data.
///
/// # Motivation
///
/// Two [`Expr`] instances will compare equal to each other if their values are
/// semantically the same. That means that even if the two [`Expr`] are
/// different allocations, they are still considered to be the same expression.
///
/// However, in some cases it is useful to distinguish [`Expr`] instances that
/// may contain semantically identical Wolfram expressions, but that are
/// different allocations.
///
/// For example, this type is used in `wl_parse::source_map` to give unique
/// source mappings, so that [`Expr`]s that are equal according to the
/// `PartialEq` impl for [`ExprKind`] (and whose hash values are therefore the
/// same) can be differentiated.
#[derive(Debug)]
pub struct ExprRefCmp(pub Expr);

impl Hash for ExprRefCmp {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0.inner).hash(state)
    }
}

impl PartialEq for ExprRefCmp {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0.inner, &other.0.inner)
    }
}

impl Eq for ExprRefCmp {}

// TODO: Add tests that `ExprRefCmp` is working as expected
