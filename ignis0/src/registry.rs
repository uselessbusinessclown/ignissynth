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
use crate::value::{Hash, SubstanceHash};
use crate::wire::{decode_form, Form, WireError};

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
    /// Slot bindings: name_hash → form_hash. Backing store for the
    /// IL's READSLOT and BINDSLOT opcodes. At stage-0 these are plain
    /// HashMap entries rather than the persistent trie S-07 uses in the
    /// habitat (that path requires v0.2.5-ignis0-store).
    slots: HashMap<Hash, Hash>,
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

    /// Bind a name slot to a Form hash. Overwrites any previous binding.
    ///
    /// This is the stage-0 backing store for `BINDSLOT`. At stage-0
    /// callers here must hold the kernel mutation capability out-of-band
    /// (the opcode itself guards via `EUnauthorised`); this method does
    /// not enforce that constraint because it is called by trusted Rust
    /// harness code, not by IL forms.
    pub fn bind_slot(&mut self, name_hash: Hash, form_hash: Hash) {
        self.slots.insert(name_hash, form_hash);
    }

    /// Look up the Form hash currently bound to `name_hash`.
    /// Returns `None` if the slot is unbound.
    /// This is the stage-0 backing store for `READSLOT`.
    pub fn read_slot(&self, name_hash: &Hash) -> Option<Hash> {
        self.slots.get(name_hash).copied()
    }

    /// Load a Form directly from its canonical wire bytes
    /// (`kernel/IL.md` § "Byte-exact wire grammar (v1)"), compute
    /// its content hash, and register it under that hash. Returns
    /// the hash assigned.
    ///
    /// This is the bridge between `wire::decode_form` and the
    /// CALL-resolution path: once `PARSEFORM` is wired to S-03,
    /// this becomes the end-to-end load path. Until then it gives
    /// the v0.2.3-ignis0-fp indirect-case harness a single entry
    /// point for registering Forms from their canonical bytes.
    ///
    /// Naming: the content hash is BLAKE3 over the *entire* wire
    /// byte sequence, including the trailing 32-byte self-hash.
    /// This is the hash any reasonable sealer would assign, and
    /// it matches what CALL expects at dispatch time.
    pub fn register_wire(&mut self, name: &str, bytes: &[u8]) -> Result<Hash, WireError> {
        let form: Form = decode_form(bytes)?;
        let hash = SubstanceHash(*blake3::hash(bytes).as_bytes());
        let loaded = LoadedForm {
            code: form.code,
            locals_n: form.locals_n as usize,
            name: name.to_string(),
        };
        self.register(hash, loaded);
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{TrapKind, Value};
    use crate::wire::{encode_form, Form};

    #[test]
    fn register_wire_round_trip_through_registry() {
        // The A9.3 canonical F, encoded to wire bytes, decoded,
        // and registered. A subsequent lookup by the returned
        // hash must produce a LoadedForm whose code matches.
        let form = Form {
            type_tag: "Form/v1".to_string(),
            arity: 1,
            locals_n: 1,
            declared_caps: vec![],
            declared_traps: vec![TrapKind::EType("Nat".into())],
            code: vec![
                Opcode::Store(0),
                Opcode::Load(0),
                Opcode::Push(Value::Nat(1)),
                Opcode::Add,
                Opcode::Ret,
            ],
        };
        let bytes = encode_form(&form).unwrap();
        let mut reg = FormRegistry::new();
        let h = reg.register_wire("canonical_F", &bytes).unwrap();
        let loaded = reg.get(&h).expect("form must be registered");
        assert_eq!(loaded.name, "canonical_F");
        assert_eq!(loaded.locals_n, 1);
        assert_eq!(loaded.code, form.code);
        assert!(reg.get_by_name("canonical_F").is_some());
    }

    #[test]
    fn register_wire_rejects_bad_bytes() {
        let mut reg = FormRegistry::new();
        let err = reg.register_wire("garbage", &[0u8; 4]).unwrap_err();
        assert!(matches!(err, WireError::Truncated));
        assert!(reg.is_empty());
    }
}
