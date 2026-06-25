//! Cheap, cross-platform memory accounting for the perf monitor.
//!
//! GPU-side numbers come from existing engine owners (glyph atlas, texture
//! pool, layer buffers/textures) via their `memory_bytes` getters; this
//! module only aggregates them. Process RSS is native-only (mach on
//! apple targets, /proc on linux/android) with no extra dependencies;
//! other targets (wasm) report `None`.

/// Byte counts collected once per frame. All fields are estimates derived
/// from resource capacities, not driver-reported allocations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryStats {
    /// Glyph atlas texture (R8Unorm, 1 byte per pixel).
    pub glyph_atlas_bytes: u64,
    /// Pooled effect/backdrop textures (grow-only `TexturePool`).
    pub texture_pool_bytes: u64,
    /// Compositor layers: vertex/index buffer capacities plus layer
    /// textures (including MSAA targets).
    pub layer_bytes: u64,
    /// Process resident set size; `None` where unsupported (wasm).
    pub process_rss_bytes: Option<u64>,
}

impl MemoryStats {
    /// Sum of the GPU-side estimates (atlas + pool + layers).
    pub fn gpu_total_bytes(&self) -> u64 {
        self.glyph_atlas_bytes + self.texture_pool_bytes + self.layer_bytes
    }
}

/// Resident set size of the current process in bytes, or `None` where the
/// platform offers no cheap query (wasm and untested targets).
pub fn process_rss_bytes() -> Option<u64> {
    imp::rss_bytes()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod imp {
    pub(super) fn rss_bytes() -> Option<u64> {
        // VmRSS is reported in kB; avoids depending on the page size.
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
        let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
        Some(kb * 1024)
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod imp {
    /// `struct mach_task_basic_info` from <mach/task_info.h>.
    #[repr(C)]
    struct MachTaskBasicInfo {
        virtual_size: u64,
        resident_size: u64,
        resident_size_max: u64,
        user_time: [i32; 2],
        system_time: [i32; 2],
        policy: i32,
        suspend_count: i32,
    }

    /// MACH_TASK_BASIC_INFO flavor id from <mach/task_info.h>.
    const MACH_TASK_BASIC_INFO: u32 = 20;
    const KERN_SUCCESS: i32 = 0;

    unsafe extern "C" {
        /// The current task's port; what the `mach_task_self()` macro reads.
        static mach_task_self_: u32;
        fn task_info(
            target_task: u32,
            flavor: u32,
            task_info_out: *mut i32,
            task_info_out_count: *mut u32,
        ) -> i32;
    }

    pub(super) fn rss_bytes() -> Option<u64> {
        let mut info = std::mem::MaybeUninit::<MachTaskBasicInfo>::uninit();
        let mut count = (size_of::<MachTaskBasicInfo>() / size_of::<u32>()) as u32;
        // SAFETY: task_info writes at most `count` natural_t words into
        // `info`, which is sized exactly for MACH_TASK_BASIC_INFO, and
        // `mach_task_self_` stays a valid task port for the process
        // lifetime.
        let kr = unsafe {
            task_info(
                mach_task_self_,
                MACH_TASK_BASIC_INFO,
                info.as_mut_ptr().cast::<i32>(),
                &mut count,
            )
        };
        if kr != KERN_SUCCESS {
            return None;
        }
        // SAFETY: the kernel returned KERN_SUCCESS, so the struct was
        // fully written.
        let info = unsafe { info.assume_init() };
        Some(info.resident_size)
    }
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios"
)))]
mod imp {
    pub(super) fn rss_bytes() -> Option<u64> {
        None
    }
}
