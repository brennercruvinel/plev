---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: changelog
---

# task-35 changelog: fix spring<t>, analytical solver

## summary
replaced forward euler integration in spring<t> with an analytical solver using pre-computed coefficients. handles all three damping regimes: under-damped (exp*sin/cos), critically damped (exp*polynomial), and over-damped (two exponentials).

## changes
- pre-computed 4 coefficients (pos_pos_coef, pos_vel_coef, vel_pos_coef, vel_vel_coef) in spring constructor
- replaced tick() with coefficient multiply-add (frame-rate independent)
- added damping_ratio() convenience method
- coefficients re-computed on set_target() when parameters change
- tests confirm identical results at 30fps vs 60fps and stability with high stiffness (10000+)
- all existing animation tests pass (zero regression)

## status
done
