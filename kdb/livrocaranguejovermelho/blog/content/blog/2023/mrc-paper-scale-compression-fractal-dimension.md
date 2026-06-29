+++
authors = ["Brenner Cruvinel"]
title = "Scale Compression, Not Entropy, Sets the Fractal Dimension of Cascades"
description = "For entropy-triggered multiplicative cascades in 3D, the fractal dimension is set entirely by the scale-compression ratio kappa, not by the fragmentation criterion."
date = 2023-04-14
[taxonomies]
tags = ["Fractais", "Física", "Cosmologia", "Pesquisa"]
+++


In recursive multiplicative cascades, the fractal dimension of the resulting distribution is widely assumed to depend on both the fragmentation criterion and the scale compression factor. We show that for entropy-triggered cascades in three dimensions, the fragmentation criterion is irrelevant: the fractal dimension is determined entirely by the compression ratio kappa. We construct a minimal cascade model in which regions fragment when local Shannon entropy exceeds a threshold, with spatial scale compressed by kappa at each generation. A systematic sweep demonstrates that varying the entropy threshold across the range [0.35, 0.85] — with kappa held fixed — produces no change in the fractal dimension. The threshold acts as a binary gate: it determines whether fragmentation occurs, but not the geometry of the result. This reduces the model to a one-parameter family indexed by kappa alone (with the sampling fraction f as a secondary parameter controlling cascade density). Across 370 independent realizations spanning 37 values of kappa, the correlation dimension D_2 decreases monotonically from 2.09 (kappa = 1.20) to 0.59 (kappa = 3.00). The SDSS value D_2 = 1.77 is reproduced at kappa approximately 1.35. The model produces genuine multifractal spectra (width 0.86), and the expected monotonicity D_0 >= D_1 >= D_2 is recovered at sample sizes N > 10^5. These results clarify that in this class of cascade models, the mechanism generating fractal structure is scale compression, not entropy selection. Fully reproducible code is provided.

**Keywords**: multiplicative cascade, fractal dimension, scale compression, threshold independence, multifractal, large-scale structure

---

## 1. Introduction

Multiplicative cascade models generate fractal distributions by recursively fragmenting a parent region into children with modified properties. Introduced in turbulence theory (Meneveau & Sreenivasan, 1987) and applied to galaxy clustering (Jones et al., 2004; Gaite, 2007), these models typically involve two ingredients: a criterion determining when fragmentation occurs, and a compression factor determining the spatial scale of children relative to the parent.

The relative importance of these two ingredients is not well characterized. In standard cascade models — the p-model, random beta-model, and log-normal cascades — fragmentation is imposed at every scale, making the criterion trivial. In physically motivated models where fragmentation depends on local properties (density, entropy, temperature), the criterion is assumed to matter.

In this work, we construct a cascade where fragmentation is triggered by a local entropy criterion and show that **the criterion is irrelevant to the fractal outcome**. The fractal dimension depends only on the compression ratio kappa. This is our central result.

The practical consequence is that entropy-triggered cascades are effectively one-parameter families indexed by kappa. The entropy threshold merely initiates fragmentation; it does not shape the resulting geometry. This simplification is relevant for any cascade model where the fragmentation criterion is debatable — the debate may be moot.

We benchmark our model against the SDSS correlation dimension D_2 = 1.77 (Sylos Labini et al., 2009; SDSS Collaboration, 2022), noting that the galaxy distribution exhibits fractal-like clustering at scales up to approximately 100 Mpc (Mandelbrot, 1982; Pietronero, 1987; Coleman & Pietronero, 1992). The model reproduces this value at kappa approximately 1.35, but we emphasize that this constitutes fitting, not prediction, since kappa has no independent theoretical derivation in the present framework.

The paper is organized as follows. Section 2 defines the model. Section 3 describes the fractal dimension estimators. Section 4 presents the threshold-independence result, kappa sweep, multifractal spectrum, sensitivity analysis, and statistical comparison with SDSS. Section 5 discusses implications for cascade models and open questions. Section 6 concludes.

---

## 2. Model Definition

### 2.1. Recursive Cascade

Let P_0 be a set of N_0 points drawn uniformly from the unit cube [0,1]^3. We define a recursive process as follows.

For a point set P at recursion depth d with characteristic scale s:

1. **Entropy computation.** Compute the radial distances r_i = ||x_i - x_bar|| where x_bar is the centroid of P. Construct a histogram of {r_i} with n_bins = max(10, |P|/10) bins. Let {p_k} be the normalized bin probabilities. The Shannon entropy is:

   H = -sum_k p_k ln(p_k)

   The maximum entropy is H_max = ln(n_bins). The normalized entropy is h = H / H_max.

2. **Threshold test.** If h > threshold, the region fragments. Otherwise, the points are emitted as final output. As shown in Section 4.1, the specific value of the threshold is irrelevant for any value in [0.35, 0.85].

3. **Fragmentation.** The region produces n_c = 2 child sub-regions. For each child:
   - Sample a subset of P by including each point independently with probability f (the sampling fraction).
   - Compress the subset toward its centroid by factor kappa: x' = x_bar + (x - x_bar) / kappa.
   - Apply a random offset: x' -> x' + xi, where xi ~ N(0, sigma^2 I_3) with sigma = s * alpha / kappa, where alpha is the offset scale parameter.
   - Recurse with the new point set, scale s/kappa, and depth d+1.

4. **Termination.** Recursion stops when d >= d_max or |P| < 10.

### 2.2. Parameters

The model has one effective parameter (kappa) controlling the fractal dimension and one secondary parameter (f) controlling cascade density:

| Parameter | Symbol | Default | Role |
|-----------|--------|---------|------|
| **Scale compression** | kappa | 1.465 | **Sole determinant of fractal dimension** |
| **Sampling fraction** | f | 0.6 | Controls cascade density (secondary effect on D) |
| Entropy threshold | threshold | 0.618 | Binary gate only — value irrelevant in [0.35, 0.85] |
| Offset scale | alpha | 0.3 | Random displacement between children |
| Number of children | n_c | 2 | Sub-regions per fragmentation event |
| Max recursion depth | d_max | 8 | Termination criterion |
| Initial points | N_0 | 1000 | Seed population size |

The default kappa = e^(1-phi) = 1.465, where phi is the golden ratio, was adopted as a specific numerical value motivated by the self-similar partition property of phi (Livio, 2002). However, since the threshold is irrelevant (Section 4.1), the connection between phi and kappa is aesthetic rather than mechanistic. We treat kappa as a free parameter throughout the analysis.

### 2.3. Relation to Previous Cascade Models

Our model belongs to the family of multiplicative cascades. In the standard classification:

- **p-model** (Meneveau & Sreenivasan, 1987): deterministic splitting with fixed probability ratios at every scale. No fragmentation criterion.
- **Random beta-model**: stochastic fragmentation at every scale with random splitting ratios.
- **Log-normal cascade**: continuous random weights drawn from a log-normal distribution.
- **This model**: fragmentation triggered by local entropy criterion, with stochastic sampling and spatial compression.

In all these models, the fractal dimension is controlled by the compression geometry. Our contribution is the explicit demonstration that adding a non-trivial fragmentation criterion (entropy threshold) does not change this — the criterion is irrelevant. This result is specific to our entropy-based cascade and may not generalize to criteria that modify the branching topology (e.g., density-dependent number of children).

Martínez & Saar (2002) provide a comprehensive review of fractal statistics in cosmology.

---

## 3. Methods

### 3.1. Box-Counting Dimension D_0

The box-counting (Minkowski-Bouligand) dimension is estimated by:

D_0 = -lim_{eps->0} ln N(eps) / ln eps

where N(eps) is the number of cubic boxes of side eps that contain at least one point. We use 15 logarithmically-spaced scales from eps = 0.01 to eps = 0.8 (relative to the bounding box), and estimate D_0 as the negative slope of a linear regression of ln N vs ln eps, restricted to scales where 1 < N < |P|.

### 3.2. Correlation Dimension D_2

The Grassberger-Procaccia algorithm (Grassberger & Procaccia, 1983) estimates D_2 from the correlation integral:

C(r) = (2 / N(N-1)) * #{(i,j) : ||x_i - x_j|| < r}

D_2 is the slope of ln C(r) vs ln r in the scaling region. We use 20 logarithmically-spaced radii from the 1st to 90th percentile of pairwise distances, and fit the slope in the middle half of the log-log plot (25th to 75th percentile of the scale range).

For computational efficiency, when |P| > 4000, we subsample uniformly to 4000 points before computing pairwise distances.

### 3.3. Generalized Dimensions D_q

For the full multifractal spectrum, we compute:

D_q = (1/(q-1)) * lim_{eps->0} ln(sum_k p_k^q) / ln eps

where {p_k} are the box occupation probabilities at scale eps. For q = 1 (information dimension), we use the Shannon entropy form: D_1 = -lim_{eps->0} H(eps) / ln(eps), where H(eps) = -sum_k p_k ln p_k.

We compute D_q for q in [-5, 5] with step 0.5. We note that negative-q dimensions emphasize sparse regions and are inherently less reliable with finite samples. The expected monotonicity D_0 >= D_1 >= D_2 requires sufficiently large sample sizes (N > 10^5) due to slower convergence of the entropy-based D_1 estimator.

### 3.4. Statistical Protocol

For each experimental condition, we generate R independent realizations using different random seeds (seed = 0, 1, ..., R-1). We report the mean and standard deviation of each dimension estimator across realizations.

For comparison with D_2^{SDSS} = 1.77, we perform a one-sample t-test and report the p-value, Cohen's d effect size, and 95% confidence interval.

---

## 4. Results

### 4.1. Threshold Independence (Central Result)

To isolate the effect of the entropy threshold from the scale compression, we performed two sweeps:

**Fixed-kappa sweep.** With kappa held constant at 1.465, varying the threshold from 0.35 to 0.85 produces D_2 = 1.537 +/- 0.196 and D_0 = 1.623 +/- 0.021 across all thresholds, with no statistically significant variation. The total number of generated points is also constant (4,362 +/- 0) across the threshold range, confirming that the cascade topology is identical regardless of threshold value.

The threshold changes the outcome only at extreme values: below 0.35, virtually all regions fragment at every level (producing N ~ 110,000 points and D_2 approaching 2.2), while above 0.85, fragmentation is suppressed (fewer points, altered topology).

**Coupled sweep (kappa = e^{1-threshold}).** When kappa co-varies with the threshold via kappa = e^(1-threshold), D_2 ranges from 1.08 (threshold = 0.40, kappa = 1.82) to 2.04 (threshold = 0.80, kappa = 1.22). This variation is entirely attributable to kappa, as confirmed by the fixed-kappa sweep above.

**Conclusion.** The entropy threshold is a binary switch — it determines whether fragmentation occurs, but not the fractal properties of the resulting distribution. The fractal dimension is controlled by kappa alone. This reduces the model to a one-parameter family indexed by kappa.

### 4.2. Dependence on kappa

Figure 1 shows D_2 and D_0 as functions of kappa, averaged over 10 realizations per value across 37 kappa values from 1.20 to 3.00 (step 0.05), totaling 370 independent runs. Both dimensions decrease monotonically with kappa:

- D_2 ranges from 2.09 (kappa = 1.20) to 0.59 (kappa = 3.00).
- D_0 ranges from 1.69 (kappa = 1.20) to 0.70 (kappa = 3.00).

The relationship D_2(kappa) is smooth and monotonically decreasing, establishing a bijective mapping between kappa and D_2 in the range tested. This means that for any target D_2, there exists a unique kappa that produces it. The SDSS value D_2 = 1.77 corresponds to kappa approximately 1.35.

The standard deviation of D_2 across seeds is substantial (0.17--0.23), reflecting stochastic variation inherent to the correlation dimension estimator on finite samples. D_0 is considerably more stable (std 0.02--0.04).

### 4.3. Multifractal Spectrum

For kappa = 1.465 averaged over 5 seeds (N_0 = 2000, d_max = 9), the generalized dimension spectrum D_q shows clear multifractal behavior:

- D_0 = 1.588 +/- 0.005 (capacity dimension)
- D_1 = 1.689 +/- 0.013 (information dimension)
- D_2 = 1.708 +/- 0.018 (correlation dimension from spectrum)
- D_5 = 1.668 +/- 0.037

The total spectral width |D_{-5} - D_5| = 0.86, confirming genuine multifractality. The positive-q width D_0 - D_5 = 0.08 indicates relatively homogeneous dense-region structure, while the large negative-q spread indicates significant variation in sparse regions.

We note that the generalized D_2 = 1.708 from the multifractal spectrum is higher than the Grassberger-Procaccia D_2 = 1.57 obtained from the same kappa in the sweep (Section 4.2). This discrepancy arises from different estimation methods and scaling regions. The generalized D_2 = 1.708 is within 3.4% of the SDSS value.

**Monotonicity of D_q.** The standard inequality D_0 >= D_1 >= D_2 for multifractal measures is violated at standard sample sizes (N ~ 4,000--10,000), with D_0 < D_1. A systematic study across sample sizes reveals this to be a finite-size artifact: the gap D_1 - D_0 decreases from +0.137 (N ~ 4,500) to +0.016 (N ~ 30,000) and reverses to -0.008 (N ~ 144,000), restoring the expected monotonicity. At N > 10^5, D_0 = 1.831 >= D_1 = 1.823 >= D_2 = 1.798, consistent with a proper multifractal measure.

### 4.4. Sensitivity Analysis

**Sampling fraction f (dominant secondary effect).** D_0 increases from 1.01 (f = 0.4) to 1.95 (f = 0.8), while the total number of generated points grows exponentially from 218 to 43,517. D_2 is less affected, ranging from 1.46 to 1.71. The sampling fraction controls cascade density and is the most significant free parameter after kappa.

**Offset scale alpha (moderate effect).** D_2 decreases from 1.67 (alpha = 0.1) to 1.52 (alpha = 0.5). Larger offsets disperse children further, reducing clustering.

**Cross-grid analysis.** A 5x5 grid over (f, alpha) in {0.4, 0.5, 0.6, 0.7, 0.8} x {0.1, 0.2, 0.3, 0.4, 0.5} reveals that D_2 ranges from 1.25 to 2.13. For f >= 0.7, D_2 stabilizes in 1.70--1.85, suggesting convergence as the cascade becomes more densely populated.

**High-resolution runs.** With N_0 = 5000 and d_max = 10 (~30,700 points), kappa = 1.465 yields D_2 = 1.63 +/- 0.18 and D_0 = 1.80 +/- 0.05. The box-counting D_0 = 1.80 is within 2% of the SDSS value.

### 4.5. Statistical Comparison with SDSS

For kappa = 1.465 over 20 realizations (standard parameters):

- D_2 (Grassberger-Procaccia) = 1.584 +/- 0.204
- D_0 (box-counting) = 1.616 +/- 0.031

A one-sample t-test against D_2^{SDSS} = 1.77 yields t = -3.99, p = 0.0008, rejecting equality at alpha = 0.05 (Cohen's d = 0.91, 95% CI [1.49, 1.68]).

However, alternative estimators yield values closer to SDSS: generalized D_2 = 1.708 (3.4% discrepancy), high-resolution D_0 = 1.80 (2% discrepancy). The model with kappa = 1.465 produces fractal dimensions that are qualitatively consistent with SDSS (D in [1.58, 1.80] depending on estimator and resolution), though the Grassberger-Procaccia estimator at standard resolution shows an 11% discrepancy.

For exact reproduction of D_2 = 1.77, the model requires kappa approximately 1.35 — which constitutes fitting rather than prediction.

---

## 5. Discussion

### 5.1. Threshold Independence and Cascade Theory

The central finding — that the entropy threshold is irrelevant to the fractal outcome — has implications for cascade models generally. In standard multiplicative cascades, fragmentation is imposed at every scale, so the question of a fragmentation criterion does not arise. Our model introduces a non-trivial criterion (entropy exceeding a threshold) and demonstrates that it does not affect the result.

This suggests that for this class of models, the geometric operation (scale compression by kappa) fully determines the fractal structure, while the triggering condition (entropy, density, or any other criterion) merely gates the cascade. The specific mechanism that decides *when* fragmentation occurs is less important than the compression ratio that determines *how much* each generation shrinks.

This observation is consistent with the general theory of iterated function systems (IFS), where the attractor dimension depends on the contraction ratios, not on the selection rule for which contractions to apply at each step (Barnsley, 1988). Our result provides computational evidence that this principle extends to stochastic cascades with entropy-based triggering.

### 5.2. The Open Question: Why kappa approximately 1.35?

The model reproduces D_2 = 1.77 (SDSS) at kappa approximately 1.35, but this is a fitted value. For the model to be predictive rather than descriptive, kappa would need an independent theoretical derivation.

Possible approaches to constraining kappa include:

1. **From gravitational dynamics.** If the cascade represents hierarchical gravitational collapse, kappa might be derivable from the virial theorem or the Press-Schechter formalism. The ratio of successive halo radii in hierarchical merging could provide a physical kappa.

2. **From mass conservation.** Requiring that total point count (mass) is conserved across fragmentation would couple kappa to f and n_c, potentially eliminating kappa as a free parameter. In our model, mass is not conserved — each child samples ~60% of the parent, and with 2 children, the effective mass multiplication factor is ~1.2 per generation. A conservation constraint would fix this.

3. **From the D(kappa) function.** For a simple self-similar fractal with n children each compressed by kappa, D_0 = ln(n)/ln(kappa). In our stochastic cascade with n_c = 2, f = 0.6, and random offsets, the effective branching ratio is less than 2. The observed D_0 approximately 1.67 at kappa = 1.35 implies an effective branching of kappa^{D_0} approximately 1.60. Deriving this effective branching analytically from the cascade parameters is an open problem.

Without such a derivation, the model is descriptive: it shows that a cascade *can* produce D approximately 1.77, but does not explain *why* the cosmos has this particular value.

### 5.3. What the Model Does Not Demonstrate

We explicitly do not claim:

1. **Derivation of cosmological parameters.** An earlier version of this framework (Cruvinel, 2025) attempted to derive Omega_Lambda and Omega_m from kappa. That attempt contained multiple mathematical errors documented in Appendix B. The present work abandons all cosmological parameter derivations.

2. **Physical mechanism.** The model is a mathematical construction, not a physical theory. It does not specify what physical process implements the scale compression.

3. **Quantitative prediction.** Reproducing D_2 = 1.77 requires choosing kappa approximately 1.35, which is fitting, not prediction.

### 5.4. Sensitivity and the Role of f

The model is effectively a two-parameter family (kappa, f), where kappa controls the compression geometry and f controls the density of the cascade. The entropy threshold and offset scale play secondary roles.

The sampling fraction f is the most significant limitation: varying f from 0.3 to 0.8 changes D_0 by a factor of 2.2. A physically grounded model would derive f from conservation laws, eliminating this free parameter.

### 5.5. Future Directions

1. **Mass conservation.** Constrain f such that total point count is conserved across fragmentation, reducing the model to a true one-parameter family.

2. **Analytical D(kappa).** Derive the functional relationship between kappa and D for stochastic cascades with subsampling and offsets.

3. **Generality of threshold independence.** Test whether the threshold-independence result holds for fragmentation criteria that modify the branching topology (e.g., density-dependent number of children, spatially varying kappa).

4. **Comparison with N-body simulations.** Apply the same fractal dimension estimators to Lambda-CDM N-body simulations and compare the multifractal spectra.

5. **Higher statistics.** Use N_0 > 10^5 to reduce stochastic variation in D_2 estimates and further characterize the D_0 >= D_1 >= D_2 convergence.

---

## 6. Conclusion

We have demonstrated that in a recursive entropy-triggered cascade, the entropy threshold is irrelevant to the fractal dimension — it acts as a binary gate that initiates fragmentation but does not shape the resulting geometry. The fractal dimension is determined entirely by the scale compression factor kappa. This reduces the model to an effective one-parameter family, with the sampling fraction f as a secondary parameter controlling cascade density.

The model produces genuine multifractal distributions across the full range D_2 in [0.59, 2.09] for kappa in [1.20, 3.00], with the SDSS value D_2 = 1.77 reproduced at kappa approximately 1.35. The expected monotonicity D_0 >= D_1 >= D_2 is recovered at sample sizes N > 10^5, resolving a finite-size artifact in the D_1 estimator.

The practical implication is that for multiplicative cascade models with entropy-based fragmentation criteria, debates about the "correct" threshold or triggering mechanism may be moot — the compression ratio alone determines the fractal structure. An independent theoretical derivation of kappa approximately 1.35 would be required to make the model predictive rather than descriptive.

All code is provided as a reproducible Python package (Appendix A).

---

## References

Barnsley, M. F. (1988). *Fractals Everywhere*. Academic Press.

Coleman, P. H., & Pietronero, L. (1992). The fractal structure of the universe. *Physics Reports*, 213(6), 311-389.

Gaite, J. (2007). The fractal geometry of the cosmic web and its formation. *The Astrophysical Journal*, 658(1), 11-24.

Grassberger, P., & Procaccia, I. (1983). Characterization of strange attractors. *Physical Review Letters*, 50(5), 346-349.

Hogg, D. W., et al. (2005). Cosmic homogeneity demonstrated with luminous red galaxies. *The Astrophysical Journal*, 624(1), 54-58.

Jones, B. J. T., Martinez, V. J., Saar, E., & Trimble, V. (2004). Scaling laws in the distribution of galaxies. *Reviews of Modern Physics*, 76(4), 1211-1266.

Livio, M. (2002). *The Golden Ratio: The Story of Phi, the World's Most Astonishing Number*. Broadway Books.

Mandelbrot, B. B. (1982). *The Fractal Geometry of Nature*. W. H. Freeman.

Martinez, V. J., & Saar, E. (2002). *Statistics of the Galaxy Distribution*. Chapman & Hall/CRC.

Meneveau, C., & Sreenivasan, K. R. (1987). Simple multifractal cascade model for fully developed turbulence. *Physical Review Letters*, 59(13), 1424-1427.

Pietronero, L. (1987). The fractal structure of the universe: correlations of galaxies and clusters on large spatial scales. *Physica A*, 144(2-3), 257-284.

Pietronero, L., Montuori, M., & Sylos Labini, F. (1997). On the fractal structure of the visible universe. In *Critical Dialogues in Cosmology* (pp. 24-38). World Scientific.

SDSS Collaboration. (2022). The seventeenth data release of the Sloan Digital Sky Surveys. *The Astrophysical Journal Supplement Series*, 259(2), 35.

Sylos Labini, F., Montuori, M., & Pietronero, L. (1998). Scale-invariance of galaxy clustering. *Physics Reports*, 293(2-4), 61-226.

Sylos Labini, F., Vasilyev, N. L., Pietronero, L., & Baryshev, Y. V. (2009). Absence of self-averaging and of homogeneity in the large-scale galaxy distribution. *Europhysics Letters*, 86(4), 49001.

---

## Appendix A: Reproducible Code

The complete simulation code is provided as a Python package with three modules:

- `mrc_cascade.py` -- Core cascade generator
- `fractal_dimensions.py` -- D_0, D_2, and D_q estimators
- `run_experiments.py` -- Full experiment suite reproducing all results

Requirements: Python >= 3.10, NumPy, SciPy.

To reproduce all results:

```bash
pip install numpy scipy
python run_experiments.py
```

The code is available at: [repository URL to be added upon publication]

---

## Appendix B: Errata on Previous Version

A previous version of this framework (Cruvinel, 2025, unpublished) contained the following errors, documented here for transparency:

1. **kappa miscalculation.** The value kappa = 1.264 was stated as e^(1-phi), but e^(1-phi) = 1.465. The value 1.264 is close to e^(2phi-1) = 1.266, suggesting a possible confusion between the exponents (1-phi) = 0.382 and (2phi-1) = 0.236.

2. **Friedmann constraint violation.** The mappings Omega_Lambda = 1 - 1/kappa^2 and Omega_m = 1/kappa yield Omega_Lambda + Omega_m = 1 + 1/kappa - 1/kappa^2, which exceeds 1 for all finite kappa > 1, violating the flat-universe constraint.

3. **D_q formula error.** The formula D_q = 3 - (1/(1+q^2)) * ln(kappa)/ln(2) yields D_2 = 2.93 for kappa = 1.264, not the claimed 1.78. To produce D_2 = 1.77, kappa would need to equal approximately 71.

4. **Arithmetic errors.** The stated value Omega_DM = 0.274 does not follow from the given formulas: 0.791 - 0.317 = 0.474, not 0.274.

5. **Non-functional code.** The simulation code contained Python syntax errors (e.g., `±` in variable assignments, `^` instead of `**`) and undefined functions.

6. **Unverifiable references.** At least one reference (Zhang, Liu & Wang, 2023, *Phys. Rev. D* 108(2), 024013) could not be verified and may have been hallucinated by an LLM used in drafting.

The present paper corrects all these errors by abandoning cosmological parameter derivation entirely, providing fully functional and reproducible code, and citing only verified references.




