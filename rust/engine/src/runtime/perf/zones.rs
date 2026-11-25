//! Named CPU zones: nested, per thread, with their start on a shared
//! clock, so a frame reads as a timeline (who ran when, inside what) and
//! as a table (where the time went, by name).
//!
//! A zone is one call at the site it measures, [`zone!`](crate::zone)
//! (or [`enter`]): it opens there and closes at the end of the enclosing
//! block. Off by default; [`enable`] turns zones on for the whole process.
//! With zones off, [`enter`] is one relaxed atomic load and a branch: no
//! clock read, no thread-local access, no lock, no allocation. With zones
//! on it reads the clock and the thread's nesting depth, and the zone is
//! recorded when it closes, complete, into its thread's buffer.
//!
//! Each thread has its own buffer, registered once in a process-wide
//! registry under the thread's name. [`take`], called once per frame by
//! the owner of the frame loop, hands over every thread's closed zones
//! since the last take; a zone still open lands in a later take. A
//! buffer holds at most [`MAX_ZONES_PER_THREAD`] records between takes,
//! the rest are counted in [`ZoneFrame::dropped`].

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use web_time::Instant;

/// Records one thread keeps between two takes; more are dropped.
pub const MAX_ZONES_PER_THREAD: usize = 8192;

/// Zones are on in the process. The only state the off path reads.
static ON: AtomicBool = AtomicBool::new(false);

/// The clock's zero: every `start_ns` counts from it.
static EPOCH: OnceLock<Instant> = OnceLock::new();

/// Every thread that recorded a zone, in registration order.
static REGISTRY: Mutex<Vec<Arc<ThreadBuffer>>> = Mutex::new(Vec::new());

/// Threads registered so far, for the name of an unnamed one.
static THREADS: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
    static BUFFER: RefCell<Option<Arc<ThreadBuffer>>> = const { RefCell::new(None) };
}

/// One closed zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneRecord {
    pub name: &'static str,
    /// Nanoseconds from the process's zone epoch to the zone's start.
    pub start_ns: u64,
    /// How long the zone was open.
    pub nanos: u64,
    /// Zones open on its thread when it opened: 0 for a top zone.
    pub depth: u32,
}

impl ZoneRecord {
    /// Nanoseconds from the epoch to the zone's end.
    pub fn end_ns(&self) -> u64 {
        self.start_ns.saturating_add(self.nanos)
    }
}

/// One thread's closed zones, in the order they closed (an inner zone
/// before the one around it).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThreadZones {
    pub name: String,
    pub zones: Vec<ZoneRecord>,
}

/// Every thread's zones closed between two takes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ZoneFrame {
    /// Threads that closed a zone, in the order they first recorded one.
    pub threads: Vec<ThreadZones>,
    /// Zones not kept because their thread's buffer was full.
    pub dropped: u64,
}

/// One zone name's cost across every thread of a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZoneStat {
    pub name: &'static str,
    pub calls: u64,
    /// Time inside the zone, the zones nested in it included.
    pub total_ns: u64,
    /// `total_ns` minus the zones nested directly inside it on its thread.
    pub self_ns: u64,
    pub min_ns: u64,
    pub max_ns: u64,
}

impl ZoneFrame {
    /// Nothing was recorded.
    pub fn is_empty(&self) -> bool {
        self.threads.is_empty() && self.dropped == 0
    }

    /// Zones recorded, every thread.
    pub fn len(&self) -> usize {
        self.threads.iter().map(|t| t.zones.len()).sum()
    }

    /// Per zone name, across threads; the most self time first (ties by
    /// name).
    pub fn stats(&self) -> Vec<ZoneStat> {
        let mut by_name: HashMap<&'static str, ZoneStat> = HashMap::new();
        for thread in &self.threads {
            let children = child_nanos(&thread.zones);
            for (zone, inner) in thread.zones.iter().zip(children) {
                let stat = by_name.entry(zone.name).or_insert(ZoneStat {
                    name: zone.name,
                    calls: 0,
                    total_ns: 0,
                    self_ns: 0,
                    min_ns: u64::MAX,
                    max_ns: 0,
                });
                stat.calls += 1;
                stat.total_ns += zone.nanos;
                stat.self_ns += zone.nanos.saturating_sub(inner);
                stat.min_ns = stat.min_ns.min(zone.nanos);
                stat.max_ns = stat.max_ns.max(zone.nanos);
            }
        }
        let mut stats: Vec<ZoneStat> = by_name.into_values().collect();
        stats.sort_by(|a, b| b.self_ns.cmp(&a.self_ns).then(a.name.cmp(b.name)));
        stats
    }
}

/// For each zone of one thread, the time of the zones nested directly
/// inside it. A zone's parent is the latest zone one level up that
/// started before it and contains it; a parent still open at the take
/// is not in the list, and its children subtract from nothing.
fn child_nanos(zones: &[ZoneRecord]) -> Vec<u64> {
    let mut order: Vec<usize> = (0..zones.len()).collect();
    order.sort_by_key(|&i| (zones[i].start_ns, zones[i].depth));
    let mut inner = vec![0u64; zones.len()];
    // the latest zone seen at each depth, in start order
    let mut last_at: Vec<Option<usize>> = Vec::new();
    for i in order {
        let zone = zones[i];
        let depth = zone.depth as usize;
        if let Some(parent) = depth
            .checked_sub(1)
            .and_then(|up| last_at.get(up).copied().flatten())
        {
            let p = zones[parent];
            if p.start_ns <= zone.start_ns && zone.end_ns() <= p.end_ns() {
                inner[parent] += zone.nanos;
            }
        }
        if last_at.len() <= depth {
            last_at.resize(depth + 1, None);
        }
        last_at[depth] = Some(i);
        // a new zone at this depth ends every deeper chain before it
        last_at.truncate(depth + 1);
    }
    inner
}

/// A thread's records since the last take.
struct ThreadBuffer {
    name: String,
    records: Mutex<Vec<ZoneRecord>>,
    dropped: AtomicU64,
}

impl ThreadBuffer {
    fn push(&self, record: ZoneRecord) {
        let mut records = lock(&self.records);
        if records.len() < MAX_ZONES_PER_THREAD {
            records.push(record);
        } else {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// A lock that survives a thread that panicked holding it: the records
/// are plain values, never left half written.
pub(super) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn epoch() -> Instant {
    *EPOCH.get_or_init(Instant::now)
}

/// Now on the zones' clock: nanoseconds from the epoch, the unit of
/// [`ZoneRecord::start_ns`]. What a message is stamped with, to sit among
/// the zones of its frame.
pub(super) fn now_ns() -> u64 {
    u64::try_from(Instant::now().duration_since(epoch()).as_nanos()).unwrap_or(u64::MAX)
}

/// Zones are on in the process.
#[inline]
pub fn is_enabled() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Turn zones on or off for every thread. Turning them off drops what
/// every thread recorded since the last take; a zone open across it is
/// not recorded.
pub fn enable(on: bool) {
    if on {
        epoch();
    }
    if ON.swap(on, Ordering::Relaxed) && !on {
        take();
    }
}

/// Open a zone named `name` until the returned guard drops. Inert when
/// zones are off: then this is the [`is_enabled`] load and nothing else
/// (no clock, no thread-local, no lock), which
/// `perf::tests::zones` checks.
#[inline]
pub fn enter(name: &'static str) -> Zone {
    if !is_enabled() {
        return Zone { open: None };
    }
    let depth = DEPTH.with(|d| {
        let depth = d.get();
        d.set(depth + 1);
        depth
    });
    let start = Instant::now();
    Zone {
        open: Some(Open { name, start, depth }),
    }
}

/// Every thread's zones closed since the last take; the next take starts
/// from here. Threads that closed nothing are left out.
pub fn take() -> ZoneFrame {
    let mut frame = ZoneFrame::default();
    let mut registry = lock(&REGISTRY);
    for buffer in registry.iter() {
        let zones = {
            let mut records = lock(&buffer.records);
            let keep = records.capacity().min(records.len().max(64));
            std::mem::replace(&mut *records, Vec::with_capacity(keep))
        };
        frame.dropped += buffer.dropped.swap(0, Ordering::Relaxed);
        if !zones.is_empty() {
            frame.threads.push(ThreadZones {
                name: buffer.name.clone(),
                zones,
            });
        }
    }
    // a thread that ended holds its buffer no more: forget it once drained
    registry.retain(|b| Arc::strong_count(b) > 1);
    frame
}

/// A zone open since [`enter`], recorded when it drops.
#[must_use = "a zone times the scope it lives in"]
pub struct Zone {
    open: Option<Open>,
}

struct Open {
    name: &'static str,
    start: Instant,
    depth: u32,
}

impl Drop for Zone {
    #[inline]
    fn drop(&mut self) {
        if let Some(open) = self.open.take() {
            close(open);
        }
    }
}

fn close(open: Open) {
    let nanos = u64::try_from(open.start.elapsed().as_nanos()).unwrap_or(u64::MAX);
    DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    if !is_enabled() {
        return;
    }
    let start_ns = u64::try_from(open.start.duration_since(epoch()).as_nanos()).unwrap_or(0);
    let record = ZoneRecord {
        name: open.name,
        start_ns,
        nanos,
        depth: open.depth,
    };
    // `try_with`: a zone closed while the thread's locals are torn down
    // is lost, never a panic
    let _ = BUFFER.try_with(|slot| {
        let mut slot = slot.borrow_mut();
        let buffer = slot.get_or_insert_with(register);
        buffer.push(record);
    });
}

/// This thread's buffer, entered into the registry.
fn register() -> Arc<ThreadBuffer> {
    let n = THREADS.fetch_add(1, Ordering::Relaxed);
    let name = std::thread::current()
        .name()
        .map_or_else(|| format!("thread-{n}"), str::to_owned);
    let buffer = Arc::new(ThreadBuffer {
        name,
        records: Mutex::new(Vec::new()),
        dropped: AtomicU64::new(0),
    });
    lock(&REGISTRY).push(Arc::clone(&buffer));
    buffer
}

/// Open a zone named `$name` (a `&'static str`) until the end of the
/// enclosing block. See `engine::runtime::perf::zones`.
#[macro_export]
macro_rules! zone {
    ($name:expr) => {
        let _zone = $crate::runtime::perf::zones::enter($name);
    };
}

/// Zones are process wide: every test that turns them on or reads them
/// holds this, so two tests never see each other's switch.
#[cfg(test)]
pub(crate) fn test_lock() -> MutexGuard<'static, ()> {
    static TESTS: Mutex<()> = Mutex::new(());
    lock(&TESTS)
}

/// Run `f` holding the registry lock, for the test that proves the off
/// path never takes it.
#[cfg(test)]
pub(super) fn holding_the_registry<R>(f: impl FnOnce() -> R) -> R {
    let _registry = lock(&REGISTRY);
    f()
}
