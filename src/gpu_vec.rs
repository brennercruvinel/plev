// ---------------------------------------------------------------------------
// Persistent GPU buffer — grows, never shrinks, partial writes
// ---------------------------------------------------------------------------

pub struct GpuVec {
    buffer: wgpu::Buffer,
    capacity: u64,
    usage: wgpu::BufferUsages,
    label: &'static str,
}

impl GpuVec {
    pub fn new(
        device: &wgpu::Device,
        label: &'static str,
        usage: wgpu::BufferUsages,
        initial_cap: u64,
    ) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: initial_cap,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            buffer,
            capacity: initial_cap,
            usage,
            label,
        }
    }

    pub fn ensure_capacity(&mut self, device: &wgpu::Device, needed: u64) {
        if needed <= self.capacity {
            return;
        }
        let new_cap = (self.capacity * 2).max(needed);
        self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(self.label),
            size: new_cap,
            usage: self.usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.capacity = new_cap;
        log::debug!("{} grew to {} bytes", self.label, new_cap);
    }

    pub fn upload<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[T],
    ) {
        let bytes = bytemuck::cast_slice(data);
        self.ensure_capacity(device, bytes.len() as u64);
        queue.write_buffer(&self.buffer, 0, bytes);
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Allocated GPU bytes (capacity, not the live byte count: the buffer
    /// grows and never shrinks). Feeds the perf monitor's memory stats.
    pub fn capacity_bytes(&self) -> u64 {
        self.capacity
    }
}
