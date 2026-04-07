//! In-memory substance store.
//!
//! This is an abstract S-03 implementation for the scaffold's
//! needs. It does not implement the full persistent hash trie
//! from `../../kernel/forms/S-03-substance-store.form`; instead
//! it uses a plain `HashMap<Hash, Cell>` and tracks pin counts
//! in the same structure.
//!
//! A proper `ignis0` would implement the trie exactly so the
//! `digest` operation produces a hash compatible with S-03's
//! obligation 5 (digest substitutivity). This scaffold's
//! `digest` is a placeholder.

use std::collections::HashMap;

use crate::value::{Hash, TrapKind, Value};

/// A sealed cell in the store.
#[derive(Debug, Clone)]
pub struct Cell {
    pub type_tag: String,
    pub value: Value,
    pub pin_count: u64,
}

/// The store itself.
pub struct SubstanceStore {
    cells: HashMap<Hash, Cell>,
}

impl SubstanceStore {
    pub fn new() -> Self {
        Self { cells: HashMap::new() }
    }

    /// Seal a value under a type tag. Idempotent: if the hash
    /// already exists, increment `pin_count` rather than
    /// inserting. Returns the content hash.
    ///
    /// The hash is computed from a canonical serialisation of
    /// `(type_tag, value)`. For this scaffold the serialisation
    /// uses a hand-rolled deterministic encoder; a real `ignis0`
    /// would use the shared canonicaliser from
    /// `../../kernel/forms/helpers/canon-normalise.form`.
    pub fn seal(&mut self, type_tag: &str, value: Value) -> Hash {
        let bytes = canonical_bytes(type_tag, &value);
        let h = Hash::of(&bytes);
        self.cells
            .entry(h)
            .and_modify(|c| c.pin_count += 1)
            .or_insert(Cell { type_tag: type_tag.to_string(), value, pin_count: 1 });
        h
    }

    /// Read a cell's value. Traps `EUNHELD` if absent.
    pub fn read(&self, h: &Hash) -> Result<Value, TrapKind> {
        self.cells
            .get(h)
            .map(|c| c.value.clone())
            .ok_or_else(|| TrapKind::EUnheld(format!("no cell at {}", h.short())))
    }

    /// Pin a cell (increment pin_count).
    pub fn pin(&mut self, h: &Hash) -> Result<(), TrapKind> {
        self.cells
            .get_mut(h)
            .map(|c| c.pin_count += 1)
            .ok_or_else(|| TrapKind::EUnheld(format!("pin: no cell at {}", h.short())))
    }

    /// Unpin a cell. If pin_count reaches zero, remove the cell.
    /// A real implementation would append a `Reclaimed{h}` entry
    /// to the weave in the same atomic call; the scaffold
    /// doesn't have a weave yet.
    pub fn unpin(&mut self, h: &Hash) -> Result<(), TrapKind> {
        let cell = self
            .cells
            .get_mut(h)
            .ok_or_else(|| TrapKind::EUnheld(format!("unpin: no cell at {}", h.short())))?;
        if cell.pin_count == 0 {
            return Err(TrapKind::EUnderflow);
        }
        cell.pin_count -= 1;
        if cell.pin_count == 0 {
            self.cells.remove(h);
        }
        Ok(())
    }

    /// Placeholder digest. A real implementation would return
    /// the root hash of the persistent trie from S-03.
    pub fn digest(&self) -> Hash {
        // Deterministic: hash the sorted cell hashes together.
        let mut keys: Vec<&Hash> = self.cells.keys().collect();
        keys.sort_by_key(|h| h.0);
        let mut buf = Vec::with_capacity(32 * keys.len());
        for k in keys {
            buf.extend_from_slice(&k.0);
        }
        Hash::of(&buf)
    }

    /// Number of live cells (diagnostic only).
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }
}

impl Default for SubstanceStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Scaffold canonical serialiser. Enough to make sealing
/// deterministic for the fixed-point test; not the real
/// canonicaliser.
fn canonical_bytes(type_tag: &str, value: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(type_tag.as_bytes());
    out.push(0u8);
    encode_value(&mut out, value);
    out
}

fn encode_value(out: &mut Vec<u8>, value: &Value) {
    match value {
        Value::Unit => out.push(0),
        Value::Bool(b) => {
            out.push(1);
            out.push(*b as u8);
        }
        Value::Nat(n) => {
            out.push(2);
            out.extend_from_slice(&n.to_be_bytes());
        }
        Value::Hash(h) => {
            out.push(3);
            out.extend_from_slice(&h.0);
        }
        Value::Bytes(b) => {
            out.push(4);
            out.extend_from_slice(&(b.len() as u64).to_be_bytes());
            out.extend_from_slice(b);
        }
        Value::Pair(a, b) => {
            out.push(5);
            encode_value(out, a);
            encode_value(out, b);
        }
        Value::Vec(vs) => {
            out.push(6);
            out.extend_from_slice(&(vs.len() as u64).to_be_bytes());
            for v in vs {
                encode_value(out, v);
            }
        }
        Value::Cell(h) => {
            out.push(7);
            out.extend_from_slice(&h.0);
        }
        Value::Cont(h) => {
            out.push(8);
            out.extend_from_slice(&h.0);
        }
    }
}
