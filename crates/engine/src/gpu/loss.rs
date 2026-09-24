//! The device-lost signal.
//!
//! wgpu reports a lost device through one callback per device
//! (`Device::set_device_lost_callback`), fired from whatever thread meets
//! the loss: a driver error or a GPU that went away fires it at once
//! (`DeviceLostReason::Unknown`, with the driver's message); a
//! `Device::destroy` fires it from the next `poll` once the queue is
//! empty (`Destroyed`, no message). Nothing else says the device is gone:
//! every later `create_*` and every submit fails silently, and the first
//! use of a dead resource that wgpu still validates is a fatal error.
//!
//! [`DeviceLoss`] is installed on every device a `GpuContext` builds and
//! turns that callback into a flag the frame reads without blocking. A
//! `Destroyed` the engine asked for itself (the context's own drop) is
//! not a loss.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::context::GpuContext;

/// Why the device was lost, as wgpu reported it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LostDevice {
    pub reason: wgpu::DeviceLostReason,
    /// The driver's message; empty for a `Destroyed`.
    pub message: String,
}

#[derive(Default)]
struct State {
    lost: AtomicBool,
    /// The engine is taking this device down itself: a `Destroyed` is
    /// expected and not a loss.
    expected: AtomicBool,
    /// A failed recovery was logged already; the next frames stay quiet.
    reported: AtomicBool,
    detail: Mutex<Option<LostDevice>>,
}

pub(crate) struct DeviceLoss(Arc<State>);

impl DeviceLoss {
    /// Install the lost callback on `device` and hand back the flag it
    /// raises. One per device: wgpu keeps a single callback slot.
    pub(crate) fn install(device: &wgpu::Device) -> Self {
        let state = Arc::new(State::default());
        let seen = Arc::clone(&state);
        device.set_device_lost_callback(move |reason, message| {
            if reason == wgpu::DeviceLostReason::Destroyed && seen.expected.load(Ordering::Acquire)
            {
                return;
            }
            *lock(&seen.detail) = Some(LostDevice { reason, message });
            seen.lost.store(true, Ordering::Release);
        });
        Self(state)
    }

    pub(crate) fn is_lost(&self) -> bool {
        self.0.lost.load(Ordering::Acquire)
    }

    pub(crate) fn detail(&self) -> Option<LostDevice> {
        lock(&self.0.detail).clone()
    }

    /// The engine is destroying this device on purpose.
    pub(crate) fn expect_destroy(&self) {
        self.0.expected.store(true, Ordering::Release);
    }

    /// `true` for the first caller per loss; the rest stay quiet.
    pub(crate) fn first_report(&self) -> bool {
        !self.0.reported.swap(true, Ordering::AcqRel)
    }
}

/// A poisoned lock only means a thread panicked while holding it; the
/// detail inside is plain data and still worth reading.
fn lock(m: &Mutex<Option<LostDevice>>) -> MutexGuard<'_, Option<LostDevice>> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl GpuContext {
    /// Whether wgpu reported this context's device lost. A flag the
    /// callback raised; reading it never blocks.
    pub fn device_lost(&self) -> bool {
        self.loss.is_lost()
    }

    /// Why the device was lost, while it is; `None` on a live device.
    pub fn lost_device(&self) -> Option<LostDevice> {
        self.loss.detail()
    }

    /// Poll the device once without waiting, so a pending `Destroyed` is
    /// delivered, then read the flag. What a frame asks before it draws.
    pub fn poll_device_loss(&self) -> bool {
        // the poll's own result says nothing about the loss; the flag does
        let _ = self.device.poll(wgpu::PollType::Poll);
        self.loss.is_lost()
    }

    /// Whether this loss is still unreported; the first call per loss
    /// answers `true`, so a failed recovery is logged once, not per frame.
    pub(crate) fn first_loss_report(&self) -> bool {
        self.loss.first_report()
    }
}

impl Drop for GpuContext {
    fn drop(&mut self) {
        // the device goes down with the context: a `Destroyed` delivered
        // after this is the engine's own doing, not a loss
        self.loss.expect_destroy();
    }
}
