//! In-memory registry of loadable Forms, keyed by content hash.
//!
//! The IL's CALL opcode names a callee by its substance hash.
//! A real `ignis0` would resolve that hash by asking the
//! substance store for a `ParsedForm/v1` cell and decoding its
//! code vec. The scaffold short-circuits that chain: Forms are
//! registered explicitly with a pre-decoded `Vec<Opcode>` and a
//! `locals_n`, and CALL looks them up directly.
//!
//! When the canonical parser lands and `PARSEFORM` is wired to
//! S-03, this registry becomes redundant — it will be replaced
//! by a direct `store.read(h)` + decode path.

use std::collections::HashMap;

use crate::opcode::Opcode;
use crate::value::Hash;

/// A loaded Form ready for execution.
#[derive(Clone, Debug)]
pub struct LoadedForm {
    pub code: Vec<Opcode>,
    pub locals_n: usize,
    /// Human-readable name, for diagnostics only. Never used for
    /// dispatch — dispatch is by `Hash`.
    pub name: String,
}

/// Content-addressed Form lookup table.
#[derive(Default)]
pub struct FormRegistry {
    forms: HashMap<Hash, LoadedForm>,
    by_name: HashMap<String, Hash>,
}

impl FormRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a Form under the given hash. Overwrites any
    /// previous entry at the same hash (registries are caches,
    /// not weaves — idempotent replacement is fine).
    pub fn register(&mut self, hash: Hash, form: LoadedForm) {
        self.by_name.insert(form.name.clone(), hash);
        self.forms.insert(hash, form);
    }

    /// Look up a Form by hash.
    pub fn get(&self, hash: &Hash) -> Option<&LoadedForm> {
        self.forms.get(hash)
    }

    /// Convenience: look up by symbolic name. Diagnostic only.
    pub fn get_by_name(&self, name: &str) -> Option<(&Hash, &LoadedForm)> {
        let h = self.by_name.get(name)?;
        self.forms.get(h).map(|f| (h, f))
    }

    pub fn len(&self) -> usize {
        self.forms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.forms.is_empty()
    }
}
