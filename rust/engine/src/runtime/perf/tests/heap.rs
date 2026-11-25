//! The counted heap: blocks and bytes through a `Counting` allocator,
//! only while on. The test binary's own allocator is not counted, so the
//! counters move only for the blocks a test asks a `Counting` for. The
//! switch is process wide: every test holds `zones::test_lock`.

use std::alloc::{GlobalAlloc, Layout};

use super::super::heap::{self, Counting, HeapTotals};
use super::super::zones;

#[test]
fn blocks_and_bytes_are_counted_only_while_on() {
    let _lock = zones::test_lock();
    let counted = Counting::system();
    let small = Layout::from_size_align(64, 8).unwrap();
    let large = Layout::from_size_align(128, 8).unwrap();

    heap::enable(false);
    let before = heap::totals();
    // SAFETY: a block of a valid, non-zero layout, given back with it.
    unsafe {
        let block = counted.alloc(small);
        assert!(!block.is_null());
        counted.dealloc(block, small);
    }
    assert_eq!(heap::totals(), before, "counted while off");

    heap::enable(true);
    // SAFETY: the block is resized from the layout it was asked with, and
    // given back with the layout of its new size.
    unsafe {
        let block = counted.alloc_zeroed(small);
        assert!(!block.is_null());
        let block = counted.realloc(block, small, large.size());
        assert!(!block.is_null());
        counted.dealloc(block, large);
    }
    heap::enable(false);

    let moved = heap::totals().since(&before);
    assert!(moved.counted);
    // asked for 64, then the resize: 64 given back and 128 asked for
    assert_eq!((moved.allocs, moved.frees), (2, 2));
    assert_eq!((moved.bytes_in, moved.bytes_out), (192, 192));
    assert_eq!(moved.net_bytes(), 0);
}

#[test]
fn live_bytes_go_below_zero_when_older_blocks_are_given_back() {
    let totals = HeapTotals {
        allocs: 1,
        frees: 3,
        bytes_in: 10,
        bytes_out: 30,
    };
    assert_eq!(totals.live_bytes(), -20);
    let frame = totals.since(&HeapTotals::default());
    assert_eq!(frame.live_bytes, -20);
    assert_eq!(frame.net_bytes(), -20);
    assert!(!HeapTotals::default().since(&HeapTotals::default()).counted);
}
