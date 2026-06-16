---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: changelog
---

# task-37 changelog: animation enhancements

## summary
added keyframesequence, repeat/reverse/delay support, step/hold easing, const-generic interpolate, and animationstate. five patterns (c2-c6) from keyframe/mina implemented.

## changes

### c5: const-generic array interpolate
- replaced 3 manual interpolate impls for [f32;2/3/4] with single const-generic impl
- replaced 3 manual springinterpolate impls with single const-generic impl

### c6: step/hold easing
- added easing::step (snaps at t=0.5) and easing::hold (returns 0.0, snaps to 1.0 at t=1.0)
- match arms added in easing::apply()

### c4: tween repeat/reverse/delay
- added delay, repeat (none/times(u32)/infinite), and reverse fields to tween
- implemented skip-during-delay, cycle counting, ping-pong logic in tick()
- builder methods: with_delay(), with_repeat(), with_reverse()

### c2: keyframesequence
- keyframesequence<t: interpolate> with vec<keyframe<t>>
- each keyframe has value, timestamp (0.0-1.0), and easing
- advance_by(), advance_and_reverse(), advance_and_wrap() methods
- now() returns interpolated value at current position
- builder pattern for construction

### c3: animationstate
- animationstate<s: hash+eq, t: interpolate> maps states to tweens/keyframesequences
- set_state() does blend-on-transition

## status
done
