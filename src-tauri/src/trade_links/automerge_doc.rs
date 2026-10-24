//! Reads the trade links out of the Automerge metadata document, where they
//! were kept before TASK-133, without the automerge crate: only the opt-in
//! sync build links that. It decodes just enough of the binary format
//! (<https://automerge.org/automerge-binary-format-spec/>) to walk the ops of
//! a saved document — the chunk header, the op columns and their encodings —
//! and resolves each map key the way Automerge does: of the ops still visible
//! for a key, the one with the greatest op id wins.
//!
//! It only reads, and anything it does not understand is an error rather than
//! a guess, so a migration never drops a link silently.

use super::TradeLink;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::io::Read;

const MAGIC: [u8; 4] = [0x85, 0x6f, 0x4a, 0x83];
const DOCUMENT_CHUNK: u8 = 0;

// Op column specs (column id << 4 | column type), deflate bit cleared.
const OBJ_ACTOR: u32 = 0x01;
const OBJ_COUNTER: u32 = 0x02;
const KEY_STRING: u32 = 0x15;
const ID_ACTOR: u32 = 0x21;
const ID_COUNTER: u32 = 0x23;
const ACTION: u32 = 0x42;
const VALUE_META: u32 = 0x56;
const VALUE: u32 = 0x57;
const SUCC_COUNT: u32 = 0x80;
const DEFLATE: u32 = 0x08;

// Op actions that give a key a value; `MAKE_*` start a nested object.
const MAKE_MAP: u64 = 0;
const PUT: u64 = 1;
const MAKE_LIST: u64 = 2;
const MAKE_TEXT: u64 = 4;
const MAKE_TABLE: u64 = 6;

/// The links in the document's root `trade_links` map; none when it has no
/// such map.
pub(super) fn trade_links(bytes: &[u8]) -> Result<Vec<TradeLink>, String> {
    let document = Document::parse(bytes)?;
    let ops = document.ops()?;
    let objects = visible_values(&ops);
    let Some(links) = objects
        .get(&ObjId::Root)
        .and_then(|root| root.get("trade_links"))
    else {
        return Ok(Vec::new());
    };
    let links = map_id(links, "trade_links")?;
    objects
        .get(&links)
        .map(|by_key| {
            by_key
                .iter()
                .map(|(key, op)| {
                    let link = objects
                        .get(&map_id(op, key)?)
                        .ok_or_else(|| format!("trade link {key} is an empty map"))?;
                    trade_link(key, link)
                })
                .collect()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}

fn trade_link(key: &str, fields: &BTreeMap<&str, &Op<'_>>) -> Result<TradeLink, String> {
    let field = |name: &str| {
        fields
            .get(name)
            .map(|op| &op.value)
            .ok_or_else(|| format!("trade link {key} has no {name}"))
    };
    let string = |name: &str| match field(name)? {
        Value::Str(s) => Ok(s.to_string()),
        other => Err(format!(
            "trade link {key}: {name} is {}, not a string",
            other.describe()
        )),
    };
    Ok(TradeLink {
        id: string("id")?,
        txn_id_a: string("txn_id_a")?,
        txn_id_b: string("txn_id_b")?,
        device_origin: string("device_origin")?,
        created_at: match field("created_at")? {
            Value::Int(i) => *i,
            other => {
                return Err(format!(
                    "trade link {key}: created_at is {}, not a signed integer",
                    other.describe()
                ))
            }
        },
    })
}

/// The object a map-valued key holds.
fn map_id<'a>(op: &Op<'a>, key: &str) -> Result<ObjId<'a>, String> {
    match op.action {
        MAKE_MAP | MAKE_TABLE => Ok(ObjId::Op(op.id)),
        _ => Err(format!("{key} is not a map")),
    }
}

// ── Resolving the document's state ───────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum ObjId<'a> {
    Root,
    Op(OpId<'a>),
}

/// An op id: its Lamport counter and its actor's id bytes, ordered in that
/// order, as Automerge orders them.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
struct OpId<'a> {
    counter: u64,
    actor: &'a [u8],
}

#[derive(Debug, PartialEq)]
enum Value<'a> {
    Str(&'a str),
    Int(i64),
    /// Any other scalar (null, bool, unsigned, float, bytes, counter,
    /// timestamp; null for an op that makes an object): its type code.
    Other(u8),
}

impl Value<'_> {
    fn describe(&self) -> String {
        match self {
            Value::Str(_) => "a string".to_string(),
            Value::Int(_) => "a signed integer".to_string(),
            Value::Other(code) => format!("a value of type {code}"),
        }
    }
}

#[derive(Debug)]
struct Op<'a> {
    obj: ObjId<'a>,
    /// The map key; none for a list element.
    key: Option<&'a str>,
    id: OpId<'a>,
    action: u64,
    value: Value<'a>,
    /// Whether no later op overwrote or deleted it.
    visible: bool,
}

/// Each object's map keys and the op whose value each currently holds: of
/// the visible value ops for a key, the one with the greatest id.
fn visible_values<'o, 'a>(ops: &'o [Op<'a>]) -> HashMap<ObjId<'a>, BTreeMap<&'a str, &'o Op<'a>>> {
    let mut objects: HashMap<ObjId<'a>, BTreeMap<&'a str, &'o Op<'a>>> = HashMap::new();
    for op in ops {
        let Some(key) = op.key else { continue };
        if !op.visible
            || !matches!(
                op.action,
                MAKE_MAP | PUT | MAKE_LIST | MAKE_TEXT | MAKE_TABLE
            )
        {
            continue;
        }
        let winner = objects.entry(op.obj).or_default().entry(key).or_insert(op);
        if op.id > winner.id {
            *winner = op;
        }
    }
    objects
}

// ── Decoding a saved document ────────────────────────────────────────

/// A saved document's actors and op columns (inflated where deflated).
struct Document<'a> {
    actors: Vec<&'a [u8]>,
    columns: HashMap<u32, Cow<'a, [u8]>>,
}

impl<'a> Document<'a> {
    /// The document chunk that must make up all of `bytes`.
    fn parse(bytes: &'a [u8]) -> Result<Self, String> {
        let mut file = Reader::new(bytes);
        if file.take(4)? != MAGIC {
            return Err("not an Automerge document".to_string());
        }
        let checksum = file.take(4)?;
        let header_start = file.pos;
        let chunk_type = file.byte()?;
        let length = usize::try_from(file.uleb()?).map_err(|_| "chunk too large".to_string())?;
        let contents = file.take(length)?;
        if Sha256::digest(&bytes[header_start..file.pos])[..4] != *checksum {
            return Err("the document's checksum does not match its contents".to_string());
        }
        if chunk_type != DOCUMENT_CHUNK {
            return Err(format!("chunk type {chunk_type} is not a saved document"));
        }
        if !file.is_empty() {
            return Err(
                "changes follow the saved document; only a compacted document can be read"
                    .to_string(),
            );
        }

        let mut doc = Reader::new(contents);
        let actors = (0..doc.uleb()?)
            .map(|_| {
                let len = doc.len()?;
                doc.take(len)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let heads = doc.len()?;
        doc.take(heads.checked_mul(32).ok_or("too many heads")?)?;
        let change_columns = column_layout(&mut doc)?;
        let op_columns = column_layout(&mut doc)?;
        doc.take(change_columns.iter().map(|&(_, len)| len).sum())?;
        let mut columns: HashMap<u32, Cow<'_, [u8]>> = HashMap::new();
        for (spec, len) in op_columns {
            let data = doc.take(len)?;
            let data = if spec & DEFLATE == 0 {
                Cow::Borrowed(data)
            } else {
                let mut inflated = Vec::new();
                flate2::read::DeflateDecoder::new(data)
                    .read_to_end(&mut inflated)
                    .map_err(|e| format!("cannot inflate column {spec:#x}: {e}"))?;
                Cow::Owned(inflated)
            };
            columns.insert(spec & !DEFLATE, data);
        }
        // What follows (the heads' change indexes) says nothing about the ops.
        Ok(Self { actors, columns })
    }

    /// Every op, in storage order.
    fn ops(&self) -> Result<Vec<Op<'_>>, String> {
        decode_ops(&self.columns, &self.actors)
    }
}

/// A column layout: each column's spec and byte length, in storage order.
fn column_layout(r: &mut Reader<'_>) -> Result<Vec<(u32, usize)>, String> {
    (0..r.uleb()?)
        .map(|_| {
            let spec = u32::try_from(r.uleb()?).map_err(|_| "bad column spec".to_string())?;
            Ok((spec, r.len()?))
        })
        .collect()
}

fn decode_ops<'a>(
    columns: &'a HashMap<u32, Cow<'a, [u8]>>,
    actors: &[&'a [u8]],
) -> Result<Vec<Op<'a>>, String> {
    let column = |spec: u32| Reader::new(columns.get(&spec).map(|c| &c[..]).unwrap_or_default());
    let mut obj_actor = Rle::new(column(OBJ_ACTOR), Reader::uleb);
    let mut obj_counter = Rle::new(column(OBJ_COUNTER), Reader::uleb);
    let mut key = Rle::new(column(KEY_STRING), Reader::string);
    let mut id_actor = Rle::new(column(ID_ACTOR), Reader::uleb);
    let mut id_counter = Delta::new(column(ID_COUNTER));
    let mut action = Rle::new(column(ACTION), Reader::uleb);
    let mut value_meta = Rle::new(column(VALUE_META), Reader::uleb);
    let mut values = column(VALUE);
    let mut succ_count = Rle::new(column(SUCC_COUNT), Reader::uleb);

    let actor = |index: Option<u64>| -> Result<&'a [u8], String> {
        let index = index.ok_or("op without an actor")?;
        usize::try_from(index)
            .ok()
            .and_then(|i| actors.get(i).copied())
            .ok_or_else(|| format!("actor {index} is not in the document"))
    };

    let mut ops = Vec::new();
    while !action.is_done() {
        let obj = match obj_actor.next()? {
            None => {
                obj_counter.next()?;
                ObjId::Root
            }
            Some(index) => ObjId::Op(OpId {
                counter: obj_counter.next()?.ok_or("object without a counter")?,
                actor: actor(Some(index))?,
            }),
        };
        let key = key.next()?;
        let id = OpId {
            actor: actor(id_actor.next()?)?,
            counter: u64::try_from(id_counter.next()?.ok_or("op without a counter")?)
                .map_err(|_| "negative op counter".to_string())?,
        };
        let action = action.next()?.ok_or("op without an action")?;
        let value = scalar(value_meta.next()?.unwrap_or(0), &mut values)?;
        let visible = succ_count.next()?.unwrap_or(0) == 0;
        ops.push(Op {
            obj,
            key,
            id,
            action,
            value,
            visible,
        });
    }
    Ok(ops)
}

/// The scalar a value-metadata entry describes, read from the value column.
fn scalar<'a>(meta: u64, values: &mut Reader<'a>) -> Result<Value<'a>, String> {
    let type_code = (meta & 0x0f) as u8;
    let len = usize::try_from(meta >> 4).map_err(|_| "value too large".to_string())?;
    let mut raw = Reader::new(values.take(len)?);
    Ok(match type_code {
        4 => Value::Int(raw.sleb()?),
        6 => Value::Str(
            std::str::from_utf8(raw.rest())
                .map_err(|_| "a string value is not UTF-8".to_string())?,
        ),
        other => Value::Other(other),
    })
}

// ── Column encodings ─────────────────────────────────────────────────

/// A run-length encoded column: a signed count n, then one value repeated n
/// times (n > 0), -n literal values (n < 0), or an unsigned count of nulls
/// (n = 0). Past its end a column reads as nulls.
struct Rle<'a, T> {
    r: Reader<'a>,
    read: fn(&mut Reader<'a>) -> Result<T, String>,
    run: Run<T>,
}

enum Run<T> {
    Repeat(T, u64),
    Literal(u64),
    Nulls(u64),
}

impl<'a, T: Clone> Rle<'a, T> {
    fn new(r: Reader<'a>, read: fn(&mut Reader<'a>) -> Result<T, String>) -> Self {
        Self {
            r,
            read,
            run: Run::Nulls(0),
        }
    }

    fn is_done(&self) -> bool {
        let left = match self.run {
            Run::Repeat(_, n) | Run::Literal(n) | Run::Nulls(n) => n,
        };
        left == 0 && self.r.is_empty()
    }

    fn next(&mut self) -> Result<Option<T>, String> {
        loop {
            match &mut self.run {
                Run::Repeat(value, n) if *n > 0 => {
                    *n -= 1;
                    return Ok(Some(value.clone()));
                }
                Run::Literal(n) if *n > 0 => {
                    *n -= 1;
                    return (self.read)(&mut self.r).map(Some);
                }
                Run::Nulls(n) if *n > 0 => {
                    *n -= 1;
                    return Ok(None);
                }
                _ if self.r.is_empty() => return Ok(None),
                _ => {}
            }
            let count = self.r.sleb()?;
            self.run = match count {
                n if n > 0 => Run::Repeat((self.read)(&mut self.r)?, n.unsigned_abs()),
                n if n < 0 => Run::Literal(n.unsigned_abs()),
                _ => Run::Nulls(self.r.uleb()?),
            };
        }
    }
}

/// A delta-encoded column: run-length encoded differences from the previous
/// value, starting from 0.
struct Delta<'a> {
    deltas: Rle<'a, i64>,
    value: i64,
}

impl<'a> Delta<'a> {
    fn new(r: Reader<'a>) -> Self {
        Self {
            deltas: Rle::new(r, Reader::sleb),
            value: 0,
        }
    }

    fn next(&mut self) -> Result<Option<i64>, String> {
        let Some(delta) = self.deltas.next()? else {
            return Ok(None);
        };
        self.value = self
            .value
            .checked_add(delta)
            .ok_or("delta column overflows")?;
        Ok(Some(self.value))
    }
}

/// A cursor over bytes with the format's LEB128 primitives.
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn is_empty(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&end| end <= self.bytes.len())
            .ok_or("the document is truncated")?;
        let taken = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(taken)
    }

    fn rest(&mut self) -> &'a [u8] {
        let rest = &self.bytes[self.pos.min(self.bytes.len())..];
        self.pos = self.bytes.len();
        rest
    }

    fn byte(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn uleb(&mut self) -> Result<u64, String> {
        let mut value = 0u64;
        for shift in (0..64).step_by(7) {
            let byte = self.byte()?;
            let low = u64::from(byte & 0x7f);
            if shift == 63 && low > 1 {
                return Err("LEB128 value overflows 64 bits".to_string());
            }
            value |= low << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err("LEB128 value overflows 64 bits".to_string())
    }

    fn sleb(&mut self) -> Result<i64, String> {
        let mut value = 0i64;
        for shift in (0..64).step_by(7) {
            let byte = self.byte()?;
            value |= i64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                if shift + 7 < 64 && byte & 0x40 != 0 {
                    value |= -1i64 << (shift + 7);
                }
                return Ok(value);
            }
        }
        Err("LEB128 value overflows 64 bits".to_string())
    }

    /// An unsigned LEB128 length.
    fn len(&mut self) -> Result<usize, String> {
        usize::try_from(self.uleb()?).map_err(|_| "length too large".to_string())
    }

    /// A length-prefixed UTF-8 string.
    fn string(&mut self) -> Result<&'a str, String> {
        let len = self.len()?;
        std::str::from_utf8(self.take(len)?).map_err(|_| "a key is not UTF-8".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Written by the pre-TASK-133 store (automerge 0.7.4); see
    // features/generated/trade_links_file.feature.
    const LINKS_DOC: &[u8] = include_bytes!("../../features/fixtures/trade_links_doc.automerge");
    const SYNCED_DOC: &[u8] =
        include_bytes!("../../features/fixtures/trade_links_synced_doc.automerge");
    const SYNC_ONLY_DOC: &[u8] = include_bytes!("../../features/fixtures/sync_only_doc.automerge");

    fn sorted(mut links: Vec<TradeLink>) -> Vec<TradeLink> {
        links.sort_by(|a, b| a.id.cmp(&b.id));
        links
    }

    #[test]
    fn reads_the_links_a_default_build_saved() {
        let links = sorted(trade_links(LINKS_DOC).unwrap());
        assert_eq!(
            links,
            vec![
                TradeLink {
                    id: "372924585e97".into(),
                    txn_id_a: "txn:sell-eth".into(),
                    txn_id_b: "txn:buy-usdc".into(),
                    device_origin: "device-aaaa1111".into(),
                    created_at: 1_760_000_300,
                },
                TradeLink {
                    id: "5b092dbe6f68".into(),
                    txn_id_a: "txn:sell-btc".into(),
                    txn_id_b: "txn:buy-aud".into(),
                    device_origin: "device-aaaa1111".into(),
                    created_at: 1_760_000_100,
                },
            ],
            "the deleted link is gone and the re-saved one carries its last created_at"
        );
    }

    #[test]
    fn concurrent_edits_resolve_to_the_greatest_op_id() {
        let links = trade_links(SYNCED_DOC).unwrap();
        assert_eq!(links.len(), 40);
        let relinked = links.iter().find(|l| l.txn_id_a == "txn:sell-0").unwrap();
        // Both devices re-linked this pair concurrently; Automerge keeps
        // device B's edit (the fixture's expected file says which).
        assert_eq!(relinked.device_origin, "device-bbbb2222");
        assert_eq!(relinked.created_at, 1_780_000_000);
        assert!(
            links.iter().all(|l| l.txn_id_a != "txn:sell-3"),
            "device A dropped it"
        );
        assert!(
            links.iter().any(|l| l.txn_id_a == "txn:x"),
            "device B added it"
        );
    }

    #[test]
    fn a_document_without_trade_links_has_none() {
        assert!(trade_links(SYNC_ONLY_DOC).unwrap().is_empty());
    }

    #[test]
    fn rejects_what_is_not_a_saved_document() {
        assert!(trade_links(b"doc 123")
            .unwrap_err()
            .contains("not an Automerge document"));
        assert!(trade_links(&LINKS_DOC[..40])
            .unwrap_err()
            .contains("truncated"));

        let mut corrupt = LINKS_DOC.to_vec();
        let last = corrupt.len() - 1;
        corrupt[last] ^= 0xff;
        assert!(trade_links(&corrupt).unwrap_err().contains("checksum"));

        let mut appended = LINKS_DOC.to_vec();
        appended.extend_from_slice(LINKS_DOC);
        assert!(trade_links(&appended)
            .unwrap_err()
            .contains("changes follow"));
    }

    #[test]
    fn leb128_round_trips_the_edges() {
        let mut r = Reader::new(&[0x00, 0x7f, 0x80, 0x01, 0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert_eq!(r.uleb().unwrap(), 0);
        assert_eq!(r.uleb().unwrap(), 127);
        assert_eq!(r.uleb().unwrap(), 128);
        assert_eq!(r.uleb().unwrap(), u64::from(u32::MAX));
        let mut r = Reader::new(&[0x7f, 0x80, 0x7f, 0x3f, 0xc0, 0x00]);
        assert_eq!(r.sleb().unwrap(), -1);
        assert_eq!(r.sleb().unwrap(), -128);
        assert_eq!(r.sleb().unwrap(), 63);
        assert_eq!(r.sleb().unwrap(), 64);
        let too_long = [0xff; 11];
        assert!(Reader::new(&too_long).uleb().is_err());
    }

    #[test]
    fn run_length_columns_decode_repeats_literals_and_nulls() {
        // 3 × 7, then the literals 1 2, then 2 nulls, then the end.
        let data = [0x03, 0x07, 0x7e, 0x01, 0x02, 0x00, 0x02];
        let mut column = Rle::new(Reader::new(&data), Reader::uleb);
        let mut values = Vec::new();
        while !column.is_done() {
            values.push(column.next().unwrap());
        }
        assert_eq!(
            values,
            [Some(7), Some(7), Some(7), Some(1), Some(2), None, None]
        );
        assert_eq!(
            column.next().unwrap(),
            None,
            "past its end a column is null"
        );
    }

    #[test]
    fn delta_columns_accumulate() {
        // Deltas 1, 1, 1 (a run of three), then the literal -2.
        let data = [0x03, 0x01, 0x7f, 0x7e];
        let mut column = Delta::new(Reader::new(&data));
        let values: Vec<_> = (0..4).map(|_| column.next().unwrap()).collect();
        assert_eq!(values, [Some(1), Some(2), Some(3), Some(1)]);
    }
}
