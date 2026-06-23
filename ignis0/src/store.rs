//! Substance store: the S-03 surface for `ignis0`.
//!
//! Two backends live here behind one [`Store`] trait:
//!
//! - [`SubstanceStore`] — the original `HashMap<Hash, Cell>` scaffold.
//!   Fast, simple, but its `digest` is a placeholder (it hashes the
//!   sorted key set and ignores the trie structure S-03 specifies).
//!
//! - [`TrieStore`] — a persistent hash-array-mapped trie (HAMT)
//!   implementing the node model of `../../kernel/types/Trie.md`
//!   (Bagwell layout, 5-bit chunks, bitmap branches, the
//!   canonicity-preserving single-leaf-branch collapse rule). Its
//!   `digest` is **substitutive**: two tries holding the same
//!   `(hash → cell, pin_count)` multiset have identical root hashes
//!   regardless of insertion order (S-03 obligation 5). This is the
//!   property the `v0.2.5-ignis0-store` milestone needs.
//!
//! Both backends compute keys with the same [`canonical_bytes`], so a
//! value sealed in one yields the same content hash in the other.
//!
//! ## Migration complete (`v0.2.5-ignis0-store`)
//!
//! The interpreter (`exec::Interpreter`), `fixed_point`, and the
//! `Capability` trait now use `&mut dyn Store` instead of the concrete
//! `SubstanceStore`. Either backend can be used interchangeably — the
//! default is still `SubstanceStore` (fast, simple) but `TrieStore`
//! (substitutive digest) can be dropped in without changing opcode
//! logic.

use std::collections::HashMap;
use std::rc::Rc;

use crate::value::{Hash, TrapKind, Value};

/// A sealed cell in the store.
#[derive(Debug, Clone)]
pub struct Cell {
    pub type_tag: String,
    pub value: Value,
    pub pin_count: u64,
}

/// The abstract S-03 substance-store interface.
///
/// Every method corresponds to one operation in
/// `../../kernel/forms/S-03-substance-store.form`. The trait exists so
/// the interpreter can be backed by either the scaffold [`SubstanceStore`]
/// or the persistent [`TrieStore`] (and, later, an external cold-weave
/// store) without changing opcode dispatch.
pub trait Store {
    /// Seal a value under a type tag; idempotent — a repeat seal of the
    /// same `(type_tag, value)` bumps `pin_count` instead of inserting.
    /// Returns the content hash.
    fn seal(&mut self, type_tag: &str, value: Value) -> Hash;
    /// Read a cell's value. Traps `EUNHELD` if absent.
    fn read(&self, h: &Hash) -> Result<Value, TrapKind>;
    /// Pin a cell (increment `pin_count`). Traps `EUNHELD` if absent.
    fn pin(&mut self, h: &Hash) -> Result<(), TrapKind>;
    /// Unpin a cell; at `pin_count == 0` the cell is reclaimed.
    /// Traps `EUNHELD` if absent, `EUNDERFLOW` if already at zero.
    fn unpin(&mut self, h: &Hash) -> Result<(), TrapKind>;
    /// The store digest. For [`TrieStore`] this is the substitutive
    /// trie root hash; for [`SubstanceStore`] it is a placeholder.
    fn digest(&self) -> Hash;
    /// Number of live cells (diagnostic).
    fn len(&self) -> usize;
    /// Whether the store holds no live cells.
    fn is_empty(&self) -> bool;
}

// ===========================================================================
// SubstanceStore — HashMap scaffold (unchanged behaviour)
// ===========================================================================

/// The scaffold store.
pub struct SubstanceStore {
    cells: HashMap<Hash, Cell>,
}

impl SubstanceStore {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
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
            .or_insert(Cell {
                type_tag: type_tag.to_string(),
                value,
                pin_count: 1,
            });
        h
    }

    /// Read a cell's value. Traps `EUNHELD` if absent.
    pub fn read(&self, h: &Hash) -> Result<Value, TrapKind> {
        self.cells
            .get(h)
            .map(|c| c.value.clone())
            .ok_or_else(|| absent(h))
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
    /// the root hash of the persistent trie from S-03 — that is
    /// exactly what [`TrieStore::digest`] now provides.
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

// The trait impl delegates to the inherent methods above. Inherent
// methods win Rust's name resolution over trait methods of the same
// name, so `SubstanceStore::seal(self, ..)` calls the inherent one —
// no recursion — and every existing `store.seal(..)` call site keeps
// resolving to the inherent method, unchanged.
impl Store for SubstanceStore {
    fn seal(&mut self, type_tag: &str, value: Value) -> Hash {
        SubstanceStore::seal(self, type_tag, value)
    }
    fn read(&self, h: &Hash) -> Result<Value, TrapKind> {
        SubstanceStore::read(self, h)
    }
    fn pin(&mut self, h: &Hash) -> Result<(), TrapKind> {
        SubstanceStore::pin(self, h)
    }
    fn unpin(&mut self, h: &Hash) -> Result<(), TrapKind> {
        SubstanceStore::unpin(self, h)
    }
    fn digest(&self) -> Hash {
        SubstanceStore::digest(self)
    }
    fn len(&self) -> usize {
        SubstanceStore::len(self)
    }
    fn is_empty(&self) -> bool {
        SubstanceStore::is_empty(self)
    }
}

// ===========================================================================
// TrieStore — persistent HAMT (kernel/types/Trie.md)
// ===========================================================================

/// A persistent HAMT node. Immutable; shared structurally via `Rc`,
/// so an insert that touches one path leaves every untouched subtree
/// (and its hash) intact — the persistence property of Trie.md.
enum Node {
    /// `TrieEmpty/v1`.
    Empty,
    /// `TrieLeaf/v1`: one `(key, cell)` pair. `cell.pin_count` is the
    /// leaf's pin count; `key` is the cell's content hash.
    Leaf { key: Hash, cell: Cell },
    /// `TrieBranch/v1`: up to 32 children, one per set bit of `bitmap`.
    /// The child for slot `b` lives at `children[popcount(bitmap & ((1<<b)-1))]`.
    Branch {
        bitmap: u32,
        children: Vec<Rc<Node>>,
    },
}

/// Persistent-HAMT-backed substance store with a substitutive digest.
pub struct TrieStore {
    root: Rc<Node>,
    /// Count of distinct live keys (diagnostic; tracked incrementally).
    len: usize,
}

/// Extract the 5-bit HAMT chunk of `key` at `depth`, MSB-first over the
/// 256-bit key. Depths 0..=50 yield a full 5-bit slot (0..=31); depth 51
/// yields the trailing 1-bit slot (0..=1). Two distinct 256-bit keys
/// always differ within these 52 levels, so splitting never recurses
/// past depth 51.
fn chunk(key: &Hash, depth: usize) -> usize {
    let base = depth * 5;
    let mut v = 0usize;
    for i in 0..5 {
        let b = base + i;
        if b >= 256 {
            break;
        }
        let byte = key.0[b / 8];
        let bit = (byte >> (7 - (b % 8))) & 1;
        v = (v << 1) | bit as usize;
    }
    v
}

/// `EUNHELD` for a key with no live cell. Shared by the [`TrieStore`]
/// read/pin/unpin paths so each call site stays a single short line.
fn absent(key: &Hash) -> TrapKind {
    TrapKind::EUnheld(format!("no cell at {}", key.short()))
}

impl TrieStore {
    pub fn new() -> Self {
        Self {
            root: Rc::new(Node::Empty),
            len: 0,
        }
    }

    /// Diagnostic: the pin count of a live key, or `None` if absent.
    pub fn pin_count(&self, h: &Hash) -> Option<u64> {
        let mut node = &self.root;
        let mut depth = 0usize;
        loop {
            match &**node {
                Node::Empty => return None,
                Node::Leaf { key, cell } => {
                    return if key == h { Some(cell.pin_count) } else { None };
                }
                Node::Branch { bitmap, children } => {
                    let s = chunk(h, depth);
                    let bit = 1u32 << s;
                    if bitmap & bit == 0 {
                        return None;
                    }
                    let idx = (bitmap & (bit - 1)).count_ones() as usize;
                    node = &children[idx];
                    depth += 1;
                }
            }
        }
    }

    /// Build a branch over two distinct leaves, diverging at the first
    /// differing chunk at or below `depth`. A shared chunk produces a
    /// single-child branch whose child is itself a branch — permitted
    /// by Trie.md's collapse rule (single child is fine when it is a
    /// Branch, not a Leaf).
    fn split(ak: Hash, ac: Cell, bk: Hash, bc: Cell, depth: usize) -> Rc<Node> {
        let a = chunk(&ak, depth);
        let b = chunk(&bk, depth);
        if a == b {
            let child = TrieStore::split(ak, ac, bk, bc, depth + 1);
            Rc::new(Node::Branch {
                bitmap: 1u32 << a,
                children: vec![child],
            })
        } else {
            let leaf_a = Rc::new(Node::Leaf { key: ak, cell: ac });
            let leaf_b = Rc::new(Node::Leaf { key: bk, cell: bc });
            let (children, bitmap) = if a < b {
                (vec![leaf_a, leaf_b], (1u32 << a) | (1u32 << b))
            } else {
                (vec![leaf_b, leaf_a], (1u32 << a) | (1u32 << b))
            };
            Rc::new(Node::Branch { bitmap, children })
        }
    }

    /// Seal-or-bump. Returns `(new_node, added_new_key)`.
    fn ins(node: &Rc<Node>, key: Hash, cell: Cell, depth: usize) -> (Rc<Node>, bool) {
        match &**node {
            Node::Empty => (Rc::new(Node::Leaf { key, cell }), true),
            Node::Leaf {
                key: k,
                cell: existing,
            } => {
                if *k == key {
                    let mut nc = existing.clone();
                    nc.pin_count += 1;
                    (Rc::new(Node::Leaf { key: *k, cell: nc }), false)
                } else {
                    (
                        TrieStore::split(*k, existing.clone(), key, cell, depth),
                        true,
                    )
                }
            }
            Node::Branch { bitmap, children } => {
                let s = chunk(&key, depth);
                let bit = 1u32 << s;
                let idx = (bitmap & (bit - 1)).count_ones() as usize;
                if bitmap & bit != 0 {
                    let (new_child, added) = TrieStore::ins(&children[idx], key, cell, depth + 1);
                    let mut nch = children.clone();
                    nch[idx] = new_child;
                    (
                        Rc::new(Node::Branch {
                            bitmap: *bitmap,
                            children: nch,
                        }),
                        added,
                    )
                } else {
                    let mut nch = children.clone();
                    nch.insert(idx, Rc::new(Node::Leaf { key, cell }));
                    (
                        Rc::new(Node::Branch {
                            bitmap: bitmap | bit,
                            children: nch,
                        }),
                        true,
                    )
                }
            }
        }
    }

    /// Collapse a branch after a child was removed: empty → `Empty`;
    /// a lone `Leaf` child lifts up to replace the branch (the
    /// canonicity rule); a lone `Branch` child stays wrapped.
    fn collapse(bitmap: u32, children: Vec<Rc<Node>>) -> Rc<Node> {
        match children.len() {
            0 => Rc::new(Node::Empty),
            1 if matches!(&*children[0], Node::Leaf { .. }) => children.into_iter().next().unwrap(),
            _ => Rc::new(Node::Branch { bitmap, children }),
        }
    }

    /// Decrement the pin count of `key`. Returns `(new_node, removed_key)`
    /// where `removed_key` is true iff the leaf hit zero and was reclaimed.
    fn unpin_node(node: &Rc<Node>, key: &Hash, depth: usize) -> Result<(Rc<Node>, bool), TrapKind> {
        match &**node {
            Node::Empty => Err(absent(key)),
            Node::Leaf { key: k, cell } => {
                if k == key {
                    if cell.pin_count == 0 {
                        return Err(TrapKind::EUnderflow);
                    }
                    let np = cell.pin_count - 1;
                    if np == 0 {
                        Ok((Rc::new(Node::Empty), true))
                    } else {
                        let mut nc = cell.clone();
                        nc.pin_count = np;
                        Ok((Rc::new(Node::Leaf { key: *k, cell: nc }), false))
                    }
                } else {
                    Err(absent(key))
                }
            }
            Node::Branch { bitmap, children } => {
                let s = chunk(key, depth);
                let bit = 1u32 << s;
                if bitmap & bit == 0 {
                    return Err(absent(key));
                }
                let idx = (bitmap & (bit - 1)).count_ones() as usize;
                let (new_child, removed) = TrieStore::unpin_node(&children[idx], key, depth + 1)?;
                if matches!(&*new_child, Node::Empty) {
                    let mut nch = children.clone();
                    nch.remove(idx);
                    Ok((TrieStore::collapse(bitmap & !bit, nch), removed))
                } else {
                    let mut nch = children.clone();
                    nch[idx] = new_child;
                    Ok((
                        Rc::new(Node::Branch {
                            bitmap: *bitmap,
                            children: nch,
                        }),
                        removed,
                    ))
                }
            }
        }
    }

    /// Increment the pin count of an existing `key`; `EUNHELD` if absent.
    fn pin_node(node: &Rc<Node>, key: &Hash, depth: usize) -> Result<Rc<Node>, TrapKind> {
        match &**node {
            Node::Empty => Err(absent(key)),
            Node::Leaf { key: k, cell } => {
                if k == key {
                    let mut nc = cell.clone();
                    nc.pin_count += 1;
                    Ok(Rc::new(Node::Leaf { key: *k, cell: nc }))
                } else {
                    Err(absent(key))
                }
            }
            Node::Branch { bitmap, children } => {
                let s = chunk(key, depth);
                let bit = 1u32 << s;
                if bitmap & bit == 0 {
                    return Err(absent(key));
                }
                let idx = (bitmap & (bit - 1)).count_ones() as usize;
                let new_child = TrieStore::pin_node(&children[idx], key, depth + 1)?;
                let mut nch = children.clone();
                nch[idx] = new_child;
                Ok(Rc::new(Node::Branch {
                    bitmap: *bitmap,
                    children: nch,
                }))
            }
        }
    }

    /// Hash a node canonically and return its subtree pin-sum, in one
    /// walk. The hash mirrors the Trie.md node layout closely enough to
    /// be substitutive: it is a function of the structure (which is
    /// itself a function of the key set) plus the per-leaf pin counts.
    ///
    /// Byte-exact fidelity to the `$$BLAKE3$$` immediates and the shared
    /// canonicaliser is deferred until those land; the digest here is
    /// already order-independent, which is the property S-03 needs.
    fn digest_and_pinsum(node: &Rc<Node>) -> (Hash, u64) {
        match &**node {
            Node::Empty => (Hash::of(b"TrieEmpty/v1"), 0),
            Node::Leaf { key, cell } => {
                let mut buf = Vec::with_capacity(13 + 32 + 8);
                buf.extend_from_slice(b"TrieLeaf/v1\0");
                buf.extend_from_slice(&key.0);
                buf.extend_from_slice(&cell.pin_count.to_be_bytes());
                (Hash::of(&buf), cell.pin_count)
            }
            Node::Branch { bitmap, children } => {
                let mut pin_sum = 0u64;
                let mut buf = Vec::with_capacity(14 + 4 + 8 + 32 * children.len());
                buf.extend_from_slice(b"TrieBranch/v1\0");
                buf.extend_from_slice(&bitmap.to_be_bytes());
                let mut child_hashes = Vec::with_capacity(children.len());
                for c in children {
                    let (ch, cps) = TrieStore::digest_and_pinsum(c);
                    pin_sum = pin_sum.saturating_add(cps);
                    child_hashes.push(ch);
                }
                buf.extend_from_slice(&pin_sum.to_be_bytes());
                for ch in &child_hashes {
                    buf.extend_from_slice(&ch.0);
                }
                (Hash::of(&buf), pin_sum)
            }
        }
    }
}

impl Default for TrieStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for TrieStore {
    fn seal(&mut self, type_tag: &str, value: Value) -> Hash {
        let bytes = canonical_bytes(type_tag, &value);
        let h = Hash::of(&bytes);
        let cell = Cell {
            type_tag: type_tag.to_string(),
            value,
            pin_count: 1,
        };
        let (new_root, added) = TrieStore::ins(&self.root, h, cell, 0);
        self.root = new_root;
        if added {
            self.len += 1;
        }
        h
    }

    fn read(&self, h: &Hash) -> Result<Value, TrapKind> {
        let mut node = &self.root;
        let mut depth = 0usize;
        loop {
            match &**node {
                Node::Empty => return Err(absent(h)),
                Node::Leaf { key, cell } => {
                    return if key == h {
                        Ok(cell.value.clone())
                    } else {
                        Err(absent(h))
                    };
                }
                Node::Branch { bitmap, children } => {
                    let s = chunk(h, depth);
                    let bit = 1u32 << s;
                    if bitmap & bit == 0 {
                        return Err(absent(h));
                    }
                    let idx = (bitmap & (bit - 1)).count_ones() as usize;
                    node = &children[idx];
                    depth += 1;
                }
            }
        }
    }

    fn pin(&mut self, h: &Hash) -> Result<(), TrapKind> {
        self.root = TrieStore::pin_node(&self.root, h, 0)?;
        Ok(())
    }

    fn unpin(&mut self, h: &Hash) -> Result<(), TrapKind> {
        let (new_root, removed) = TrieStore::unpin_node(&self.root, h, 0)?;
        self.root = new_root;
        if removed {
            self.len -= 1;
        }
        Ok(())
    }

    fn digest(&self) -> Hash {
        TrieStore::digest_and_pinsum(&self.root).0
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }
}

// ===========================================================================
// Shared canonical serialiser
// ===========================================================================

/// Scaffold canonical serialiser. Enough to make sealing
/// deterministic for the fixed-point test; not the real
/// canonicaliser. Shared by both store backends so a value sealed in
/// one has the same content hash in the other.
pub(crate) fn canonical_bytes(type_tag: &str, value: &Value) -> Vec<u8> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn nat(n: u128) -> Value {
        Value::Nat(n)
    }

    // A handful of distinct values whose content hashes scatter across
    // the trie. Sealing the same set in any order must yield the same
    // TrieStore digest (S-03 substitutivity).
    fn sample_keys(store: &mut TrieStore, order: &[u128]) -> Vec<Hash> {
        order.iter().map(|&n| store.seal("T/v1", nat(n))).collect()
    }

    #[test]
    fn empty_digest_is_stable() {
        let a = TrieStore::new();
        let b = TrieStore::new();
        assert_eq!(a.digest(), b.digest());
        assert!(a.is_empty());
        assert_eq!(a.digest(), Hash::of(b"TrieEmpty/v1"));
    }

    #[test]
    fn digest_is_permutation_invariant() {
        // The substitutive-digest property: identical (key, pin) multiset
        // ⇒ identical root hash, regardless of insertion order.
        let mut s1 = TrieStore::new();
        let mut s2 = TrieStore::new();
        sample_keys(&mut s1, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        sample_keys(&mut s2, &[10, 7, 3, 9, 1, 6, 4, 8, 2, 5]);
        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_eq!(s1.digest(), s2.digest());
    }

    #[test]
    fn repeated_seal_bumps_pin_count() {
        let mut s = TrieStore::new();
        let h = s.seal("T/v1", nat(42));
        assert_eq!(s.pin_count(&h), Some(1));
        let h2 = s.seal("T/v1", nat(42));
        assert_eq!(h, h2, "same value ⇒ same content hash");
        assert_eq!(s.pin_count(&h), Some(2));
        assert_eq!(s.len(), 1, "a bump is not a new key");
    }

    #[test]
    fn read_round_trips_and_traps_when_absent() {
        let mut s = TrieStore::new();
        let h = s.seal("T/v1", nat(7));
        assert_eq!(s.read(&h).unwrap(), nat(7));
        assert!(matches!(s.read(&Hash::BOTTOM), Err(TrapKind::EUnheld(_))));
    }

    #[test]
    fn unpin_reclaims_and_restores_prior_digest() {
        // seal a..; snapshot digest; seal one more; unpin it back to zero;
        // the digest must return to the snapshot (reclaim is exact) and the
        // reclaimed key must read as absent.
        let mut s = TrieStore::new();
        sample_keys(&mut s, &[1, 2, 3]);
        let d_before = s.digest();
        let extra = s.seal("T/v1", nat(99));
        assert_ne!(s.digest(), d_before);
        assert_eq!(s.len(), 4);
        s.unpin(&extra).unwrap();
        assert_eq!(s.len(), 3);
        assert_eq!(s.digest(), d_before, "reclaim returns to the prior root");
        assert!(matches!(s.read(&extra), Err(TrapKind::EUnheld(_))));
    }

    #[test]
    fn unpin_decrements_before_reclaim() {
        let mut s = TrieStore::new();
        let h = s.seal("T/v1", nat(5));
        s.seal("T/v1", nat(5)); // pin_count = 2
        s.unpin(&h).unwrap(); // pin_count = 1, still live
        assert_eq!(s.pin_count(&h), Some(1));
        assert_eq!(s.read(&h).unwrap(), nat(5));
        s.unpin(&h).unwrap(); // pin_count = 0, reclaimed
        assert_eq!(s.pin_count(&h), None);
    }

    #[test]
    fn unpin_absent_traps_eunheld() {
        let mut s = TrieStore::new();
        s.seal("T/v1", nat(1));
        assert!(matches!(s.unpin(&Hash::BOTTOM), Err(TrapKind::EUnheld(_))));
    }

    #[test]
    fn persistence_does_not_disturb_siblings() {
        // After many inserts and a removal, every surviving key still reads
        // back — structural sharing must not corrupt untouched subtrees.
        let mut s = TrieStore::new();
        let keys = sample_keys(&mut s, &(0..64u128).collect::<Vec<_>>());
        assert_eq!(s.len(), 64);
        // remove the even-indexed keys
        for (i, k) in keys.iter().enumerate() {
            if i % 2 == 0 {
                s.unpin(k).unwrap();
            }
        }
        assert_eq!(s.len(), 32);
        for (i, k) in keys.iter().enumerate() {
            if i % 2 == 0 {
                assert!(matches!(s.read(k), Err(TrapKind::EUnheld(_))));
            } else {
                assert_eq!(s.read(k).unwrap(), nat(i as u128));
            }
        }
    }

    #[test]
    fn removal_order_independence() {
        // Build the same final set two ways: insert 0..20 then remove a
        // subset, vs. insert only the survivors. Canonical form ⇒ equal digest.
        let survivors: Vec<u128> = vec![2, 3, 5, 7, 11, 13, 17, 19];
        let mut full = TrieStore::new();
        let all = sample_keys(&mut full, &(0..20u128).collect::<Vec<_>>());
        for (i, k) in all.iter().enumerate() {
            if !survivors.contains(&(i as u128)) {
                full.unpin(k).unwrap();
            }
        }
        let mut direct = TrieStore::new();
        sample_keys(&mut direct, &survivors);
        assert_eq!(full.len(), survivors.len());
        assert_eq!(
            full.digest(),
            direct.digest(),
            "collapse rule makes removal-built and insert-built tries identical"
        );
    }

    #[test]
    fn cross_backend_keys_agree() {
        // Both backends compute the same content hash for the same value,
        // and agree on membership and read-back (digests differ by design).
        let mut map = SubstanceStore::new();
        let mut trie = TrieStore::new();
        for n in 0..32u128 {
            let hm = map.seal("T/v1", nat(n));
            let ht = trie.seal("T/v1", nat(n));
            assert_eq!(hm, ht, "shared canonical_bytes ⇒ identical keys");
        }
        assert_eq!(map.len(), trie.len());
        for n in 0..32u128 {
            let h = Hash::of(&canonical_bytes("T/v1", &nat(n)));
            assert_eq!(
                <SubstanceStore as Store>::read(&map, &h).unwrap(),
                <TrieStore as Store>::read(&trie, &h).unwrap()
            );
        }
    }

    #[test]
    fn usable_as_trait_object() {
        // The seam compiles: both backends are `dyn Store`.
        fn digest_of(s: &dyn Store) -> Hash {
            s.digest()
        }
        let map = SubstanceStore::new();
        let trie = TrieStore::new();
        let _ = digest_of(&map);
        let _ = digest_of(&trie);
    }
}
