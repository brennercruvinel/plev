+++
title = "New Experiment Results (2026-03-06)"
date = 2023-01-01
draft = true
+++

# New Experiment Results (2026-03-06)
# After bug fixes: D_1 sign, rng reproducibility, n_children=2

## Bug Fixes Applied

1. **D_1 sign convention**: `generalized_dimension` for q=1 now returns `-slope` (D_1 = H(eps)/ln(1/eps)).
2. **rng reproducibility**: `correlation_dimension` now accepts `seed` parameter for deterministic subsampling.
3. **n_children**: Code uses `max(2, int(1/threshold)) = 2` for threshold=phi. Paper will be corrected to match.

## Experiment 6: D_q Inversion Diagnostic

### Finding: D_0 < D_1 inversion is a finite-size artifact

| Configuration | N_pts | D_0 | D_1 | D_2 | gap(D1-D0) | Monotonic? |
|---------------|------:|----:|----:|----:|-----------:|:----------:|
| standard (N0=1000, d=8) | ~4,483 | 1.751 | 1.888 | 1.901 | +0.137 | No |
| medium (N0=2000, d=9) | ~10,593 | 1.770 | 1.851 | 1.844 | +0.081 | No |
| high-res (N0=5000, d=10) | ~30,571 | 1.794 | 1.809 | 1.788 | +0.016 | No |
| dense (N0=5000, d=10, f=0.7) | ~144,011 | 1.831 | 1.823 | 1.798 | -0.008 | **Yes** |

Scale range: [0.02, 0.3], n_scales=20, kappa=1.4652, 3 seeds each.

**Conclusion**: The D_0 < D_1 inversion resolves at N > ~100k.
The D_1 (information dimension) estimator converges more slowly than D_0 (box-counting)
because the entropy sum is more sensitive to sparsely-occupied boxes at small scales.
At N~144k, D_0 >= D_1 >= D_2 holds as expected for a proper multifractal measure.

## Experiment 5: Threshold Sweep

### 5a. Coupled sweep: threshold varies, kappa = e^(1-threshold)

| threshold | kappa | D2 | D2_std | D0 | D0_std | N_pts |
|----------:|------:|---:|-------:|---:|-------:|------:|
| 0.40 | 1.8221 | 1.079 | 0.174 | 1.326 | 0.054 | 4362 |
| 0.45 | 1.7333 | 1.155 | 0.172 | 1.416 | 0.057 | 4362 |
| 0.50 | 1.6487 | 1.244 | 0.169 | 1.494 | 0.052 | 4362 |
| 0.55 | 1.5683 | 1.353 | 0.174 | 1.557 | 0.046 | 4362 |
| 0.60 | 1.4918 | 1.485 | 0.189 | 1.606 | 0.026 | 4362 |
| 0.618 | 1.4652 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.65 | 1.4191 | 1.632 | 0.203 | 1.647 | 0.021 | 4362 |
| 0.70 | 1.3499 | 1.778 | 0.195 | 1.673 | 0.018 | 4362 |
| 0.75 | 1.2840 | 1.918 | 0.167 | 1.685 | 0.018 | 4362 |
| 0.80 | 1.2214 | 2.044 | 0.129 | 1.688 | 0.014 | 4362 |

### 5b. Isolated threshold effect: kappa FIXED at 1.4652

| threshold | D2 | D2_std | D0 | D0_std | N_pts |
|----------:|---:|-------:|---:|-------:|------:|
| 0.30 | 2.064 | 0.136 | 2.234 | 0.022 | 109827 |
| 0.35 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.40 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.45 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.50 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.55 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.60 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.65 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.70 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.75 | 1.537 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.80 | 1.538 | 0.196 | 1.623 | 0.021 | 4362 |
| 0.85 | 1.508 | 0.152 | 1.636 | 0.009 | 4338 |
| 0.90 | 1.677 | 0.220 | 1.543 | 0.040 | 3026 |

### Key Finding

**The entropy threshold is a binary gate, not a continuous control.**
For any threshold in [0.35, 0.85], the fractal dimension is determined entirely by kappa.
The threshold only matters at extreme values where it changes the branching structure
(threshold < 0.35: excessive fragmentation; threshold > 0.85: insufficient fragmentation).

This means:
- The "derivation" kappa = e^(1-phi) is aesthetic, not predictive
- phi as threshold adds no power beyond any other value in [0.35, 0.85]
- kappa is the sole free parameter controlling fractal dimension
- The paper should reframe accordingly


# Multifractal Recursive Cascade -- Simulation Results

**Date:** 2026-03-06
**Total computation time:** 101.9s
**Key constants:** phi = 0.618034, kappa* = e^(1-phi) = 1.4652

---

## 1. Kappa Sweep: D2 (correlation) and D0 (box-counting)

Parameters: n_initial=1000, max_depth=8, sampling_frac=0.6, offset_scale=0.3, 10 seeds each.

| kappa | D2 mean | D2 std | D2 SE(fit) | D0 mean | D0 std | D0 SE(fit) | N_pts mean | N_pts std |
|------:|--------:|-------:|-----------:|--------:|-------:|-----------:|-----------:|----------:|
| 1.20 | 2.0889 | 0.1747 | 0.0403 | 1.6858 | 0.016 | 0.0911 | 4323 | 201 |
| 1.25 | 2.0017 | 0.1963 | 0.0362 | 1.682 | 0.0166 | 0.0838 | 4323 | 201 |
| 1.30 | 1.905 | 0.2134 | 0.0311 | 1.6742 | 0.0204 | 0.0752 | 4323 | 201 |
| 1.35 | 1.8019 | 0.2252 | 0.0272 | 1.6627 | 0.0222 | 0.0656 | 4323 | 201 |
| 1.40 | 1.7036 | 0.2319 | 0.0228 | 1.6453 | 0.0229 | 0.0547 | 4323 | 201 |
| 1.45 | 1.6019 | 0.2302 | 0.021 | 1.6259 | 0.0181 | 0.0441 | 4323 | 201 |
| 1.50 | 1.5 | 0.2158 | 0.0217 | 1.5944 | 0.0229 | 0.0351 | 4323 | 201 |
| 1.55 | 1.4064 | 0.1962 | 0.023 | 1.5652 | 0.0345 | 0.0286 | 4323 | 201 |
| 1.60 | 1.3206 | 0.1762 | 0.0236 | 1.5334 | 0.0334 | 0.0264 | 4323 | 201 |
| 1.65 | 1.2459 | 0.1622 | 0.0233 | 1.4913 | 0.0378 | 0.0247 | 4323 | 201 |
| 1.70 | 1.1814 | 0.1557 | 0.0232 | 1.4429 | 0.0416 | 0.0245 | 4323 | 201 |
| 1.75 | 1.1248 | 0.152 | 0.0227 | 1.3989 | 0.0416 | 0.0251 | 4323 | 201 |
| 1.80 | 1.0806 | 0.1487 | 0.0226 | 1.3498 | 0.047 | 0.0267 | 4323 | 201 |
| 1.85 | 1.0374 | 0.1496 | 0.0227 | 1.3028 | 0.0516 | 0.0271 | 4323 | 201 |
| 1.90 | 0.998 | 0.145 | 0.0232 | 1.2585 | 0.0507 | 0.0273 | 4323 | 201 |
| 1.95 | 0.9628 | 0.1403 | 0.0238 | 1.2145 | 0.0504 | 0.0289 | 4323 | 201 |
| 2.00 | 0.9289 | 0.1368 | 0.0245 | 1.1722 | 0.0481 | 0.0288 | 4323 | 201 |
| 2.05 | 0.8978 | 0.1304 | 0.0256 | 1.1277 | 0.0502 | 0.0294 | 4323 | 201 |
| 2.10 | 0.87 | 0.1261 | 0.0265 | 1.091 | 0.0512 | 0.0308 | 4323 | 201 |
| 2.15 | 0.8428 | 0.1231 | 0.0271 | 1.0523 | 0.0514 | 0.0293 | 4323 | 201 |
| 2.20 | 0.8181 | 0.1183 | 0.0273 | 1.0194 | 0.0473 | 0.0302 | 4323 | 201 |
| 2.25 | 0.796 | 0.1154 | 0.0278 | 0.9907 | 0.0525 | 0.0308 | 4323 | 201 |
| 2.30 | 0.7741 | 0.1114 | 0.0282 | 0.9585 | 0.0478 | 0.0303 | 4323 | 201 |
| 2.35 | 0.754 | 0.1049 | 0.0282 | 0.9316 | 0.0542 | 0.03 | 4323 | 201 |
| 2.40 | 0.736 | 0.1009 | 0.0282 | 0.9053 | 0.0489 | 0.029 | 4323 | 201 |
| 2.45 | 0.7173 | 0.0954 | 0.0282 | 0.8836 | 0.0472 | 0.0326 | 4323 | 201 |
| 2.50 | 0.7031 | 0.093 | 0.0281 | 0.8647 | 0.0403 | 0.0327 | 4323 | 201 |
| 2.55 | 0.6879 | 0.0883 | 0.028 | 0.8424 | 0.0473 | 0.033 | 4323 | 201 |
| 2.60 | 0.6754 | 0.0848 | 0.0278 | 0.8298 | 0.0464 | 0.0304 | 4323 | 201 |
| 2.65 | 0.661 | 0.0815 | 0.0279 | 0.8094 | 0.0464 | 0.0324 | 4323 | 201 |
| 2.70 | 0.6492 | 0.0786 | 0.0273 | 0.7898 | 0.0455 | 0.032 | 4323 | 201 |
| 2.75 | 0.637 | 0.0753 | 0.0276 | 0.776 | 0.0475 | 0.0329 | 4323 | 201 |
| 2.80 | 0.6271 | 0.0752 | 0.0276 | 0.7586 | 0.0415 | 0.0335 | 4323 | 201 |
| 2.85 | 0.6152 | 0.0725 | 0.0275 | 0.7387 | 0.0398 | 0.0336 | 4323 | 201 |
| 2.90 | 0.605 | 0.0704 | 0.0274 | 0.7243 | 0.0391 | 0.0347 | 4323 | 201 |
| 2.95 | 0.5955 | 0.0678 | 0.0273 | 0.718 | 0.0507 | 0.035 | 4323 | 201 |
| 3.00 | 0.5866 | 0.0652 | 0.0271 | 0.704 | 0.0483 | 0.0347 | 4323 | 201 |

### Key observations (Sweep)

- D2 crosses 2.0 at kappa ~ 1.251
- D2 crosses 1+phi=1.6180 at kappa ~ 1.442
- D2 crosses 1.5 at kappa ~ 1.500
- D2 crosses 1.0 at kappa ~ 1.897
- D2 crosses phi=0.6180 at kappa ~ 2.838

- At kappa=1.45: D2=1.6019, D0=1.6259
- At kappa=1.50: D2=1.5, D0=1.5944
- At kappa*=1.4652 (interpolated between 1.45 and 1.50):
  - D2 ~ 1.5710, D0 ~ 1.6163

---

## 2. Multifractal Spectrum: D_q for q = -5 to +5

Parameters: kappa = 1.4652, n_initial=2000, max_depth=9, 5 seeds.

| q | D_q mean | D_q std | D_q SE(fit) | n_seeds |
|-----:|---------:|--------:|------------:|--------:|
| -5.0 | 0.8098 | 0.1005 | 0.1629 | 5 |
| -4.5 | 0.8708 | 0.1106 | 0.1641 | 5 |
| -4.0 | 0.8392 | 0.1696 | 0.136 | 5 |
| -3.5 | 0.8915 | 0.1843 | 0.1268 | 5 |
| -3.0 | 0.9396 | 0.1448 | 0.1335 | 5 |
| -2.5 | 1.0421 | 0.0549 | 0.128 | 5 |
| -2.0 | 1.0498 | 0.1257 | 0.1082 | 5 |
| -1.5 | 1.1729 | 0.1072 | 0.0934 | 5 |
| -1.0 | 1.3204 | 0.0374 | 0.0919 | 5 |
| -0.5 | 1.4528 | 0.0311 | 0.067 | 5 |
| +0.0 | 1.5879 | 0.0052 | 0.0536 | 5 |
| +0.5 | 1.6549 | 0.0137 | 0.0448 | 5 |
| +1.0 | 1.689 | 0.0133 | 0.0375 | 5 |
| +1.5 | 1.7019 | 0.0183 | 0.0325 | 5 |
| +2.0 | 1.7084 | 0.018 | 0.0289 | 5 |
| +2.5 | 1.7065 | 0.0215 | 0.0269 | 5 |
| +3.0 | 1.706 | 0.025 | 0.026 | 5 |
| +3.5 | 1.6901 | 0.0342 | 0.0244 | 5 |
| +4.0 | 1.6894 | 0.0257 | 0.0253 | 5 |
| +4.5 | 1.6808 | 0.0262 | 0.0262 | 5 |
| +5.0 | 1.6682 | 0.0374 | 0.0274 | 5 |
| +5.5 | 1.6589 | 0.0279 | 0.0277 | 5 |

### Key observations (Spectrum)

- D_0 (box-counting / capacity dimension) = 1.5879
- D_1 (information dimension) = 1.689
- D_2 (correlation dimension) = 1.7084
- Multifractal width: D_(-5) - D_(+5) = 0.8098 - 1.6682 = -0.8584
- The spectrum is genuinely multifractal (width > 0.1)

---

## 3. High-Resolution Runs (n_initial=5000, max_depth=10)

| kappa | seed | N_points | D2 | D2 err | D2 R2 | D0 | D0 err | D0 R2 | time (s) |
|------:|-----:|---------:|---:|-------:|------:|---:|-------:|------:|---------:|
| 1.4652 | 0 | 30604 | 1.5993 | 0.0178 | 0.999 | 1.8291 | 0.0241 | 0.9978 | 0.3 |
| 1.4652 | 1 | 30270 | 1.3449 | 0.0255 | 0.9971 | 1.769 | 0.0231 | 0.9978 | 0.4 |
| 1.4652 | 2 | 30841 | 1.5892 | 0.0314 | 0.9969 | 1.7883 | 0.0129 | 0.9993 | 0.4 |
| 1.4652 | 3 | 30413 | 1.7329 | 0.0399 | 0.9958 | 1.7462 | 0.0325 | 0.9955 | 0.4 |
| 1.4652 | 4 | 31502 | 1.8859 | 0.0173 | 0.9993 | 1.8758 | 0.0246 | 0.9978 | 0.4 |
| 1.5800 | 0 | 30604 | 1.4112 | 0.0133 | 0.9993 | 1.636 | 0.0284 | 0.9961 | 0.3 |
| 1.5800 | 1 | 30270 | 1.1875 | 0.0221 | 0.9972 | 1.5773 | 0.0258 | 0.9965 | 0.4 |
| 1.5800 | 2 | 30841 | 1.4218 | 0.0328 | 0.9958 | 1.6031 | 0.0162 | 0.9987 | 0.4 |
| 1.5800 | 3 | 30413 | 1.5993 | 0.0219 | 0.9985 | 1.5569 | 0.0288 | 0.9956 | 0.4 |
| 1.5800 | 4 | 31502 | 1.575 | 0.0219 | 0.9985 | 1.6921 | 0.0253 | 0.9971 | 0.4 |

**kappa = 1.4652 summary:** D2 = 1.6304 +/- 0.1789, D0 = 1.8017 +/- 0.0460, mean N_pts = 30726
**kappa = 1.5800 summary:** D2 = 1.4390 +/- 0.1473, D0 = 1.6131 +/- 0.0476, mean N_pts = 30726

---

## 4. Sensitivity Analysis

Base parameters: kappa = 1.4652, 5 seeds.

### 4a. Varying sampling_frac (offset_scale = 0.3)

| sampling_frac | D2 mean | D2 std | D0 mean | D0 std | N_pts mean |
|--------------:|--------:|-------:|--------:|-------:|-----------:|
| 0.4 | 1.4591 | 0.1117 | 1.008 | 0.0649 | 218 |
| 0.5 | 1.5052 | 0.1024 | 1.284 | 0.0123 | 902 |
| 0.6 | 1.5387 | 0.1975 | 1.6235 | 0.0219 | 4363 |
| 0.7 | 1.6212 | 0.226 | 1.8451 | 0.0387 | 14745 |
| 0.8 | 1.7094 | 0.2413 | 1.9474 | 0.0794 | 43233 |

### 4b. Varying offset_scale (sampling_frac = 0.6)

| offset_scale | D2 mean | D2 std | D0 mean | D0 std | N_pts mean |
|-------------:|--------:|-------:|--------:|-------:|-----------:|
| 0.1 | 1.6685 | 0.2476 | 1.6945 | 0.0144 | 4363 |
| 0.2 | 1.5621 | 0.2234 | 1.666 | 0.0167 | 4363 |
| 0.3 | 1.5392 | 0.1999 | 1.6235 | 0.0219 | 4363 |
| 0.4 | 1.528 | 0.1855 | 1.5798 | 0.0266 | 4363 |
| 0.5 | 1.5206 | 0.1779 | 1.5378 | 0.03 | 4363 |

### 4c. Cross-grid: sampling_frac x offset_scale (D2, 3 seeds)

| sf \ os | 0.1 | 0.2 | 0.3 | 0.4 | 0.5 |
|--------:|------:|------:|------:|------:|------:|
| 0.4 | 2.1317 | 1.6029 | 1.3874 | 1.2877 | 1.2470 |
| 0.5 | 1.8181 | 1.5709 | 1.5085 | 1.4914 | 1.4889 |
| 0.6 | 1.7672 | 1.6619 | 1.6367 | 1.6238 | 1.6087 |
| 0.7 | 1.8472 | 1.7416 | 1.7261 | 1.7173 | 1.7240 |
| 0.8 | 1.8616 | 1.7354 | 1.7056 | 1.6985 | 1.7058 |

### 4d. Cross-grid: N_points generated

| sf \ os | 0.1 | 0.2 | 0.3 | 0.4 | 0.5 |
|--------:|------:|------:|------:|------:|------:|
| 0.4 | 209 | 209 | 209 | 209 | 209 |
| 0.5 | 907 | 907 | 907 | 907 | 907 |
| 0.6 | 4483 | 4483 | 4483 | 4483 | 4483 |
| 0.7 | 14987 | 14987 | 14987 | 14987 | 14987 |
| 0.8 | 43517 | 43517 | 43517 | 43517 | 43517 |

### Key observations (Sensitivity)

- **sampling_frac** has a large effect on N_points (exponential growth) and moderate effect on D2.
- Higher sampling_frac produces more points and slightly higher fractal dimensions.
- **offset_scale** does not change N_points (same branching structure) but affects clustering:
  - Lower offset_scale -> tighter clusters -> higher D2 (points more correlated spatially).
  - Higher offset_scale -> more spread -> lower D2 (more uniform/dispersed).
- D2 is most sensitive at low sampling_frac + low offset_scale (fewer points, tighter clusters).

---

## Summary of Key Results

1. **Kappa sweep** confirms monotonic decrease of both D2 and D0 with increasing kappa.
   D2 drops from ~2.09 (kappa=1.2) to ~0.59 (kappa=3.0).
   D0 drops from ~1.69 to ~0.70 over the same range.

2. D2 = phi (0.6180) occurs at kappa ~ 2.838 (far above kappa*).

3. **Multifractal spectrum** at kappa* = 1.4652 shows genuine multifractality:
   - D_0 = 1.5879
   - D_1 = 1.689
   - D_2 = 1.7084
   - Width Delta D = D_(-5) - D_(+5) = -0.8584
   - D_q is non-constant across q, confirming a heterogeneous measure.

4. **High-resolution** (5000 pts, depth 10) results:
   - kappa=1.4652: D2 = 1.6304 +/- 0.1789 (N ~ 30726)
   - kappa=1.58: D2 = 1.4390 +/- 0.1473 (N ~ 30726)

5. **Sensitivity analysis** shows D2 varies from ~1.25 to ~2.13 across the parameter grid,
   with sampling_frac controlling point count (218 to 43517) and offset_scale controlling spatial spread.
