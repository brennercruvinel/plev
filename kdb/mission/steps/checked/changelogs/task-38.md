---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: changelog
---

# task-38 changelog: event batching

## summary
implemented batch-drain pattern for winit events to reduce GPU work during rapid input. events are accumulated in window_event() and processed in bulk during about_to_wait() before a single render pass.

## changes
- events accumulated in vec<inputevent> in window_event() instead of immediate processing
- batch drain in about_to_wait(): all events processed, then single compositor.resolve() + render_pass
- batch limit of 50 events per iteration to prevent starvation
- no input events lost, all processed, only rendering is batched
- latency stays within 1 frame of delay maximum

## status
done
