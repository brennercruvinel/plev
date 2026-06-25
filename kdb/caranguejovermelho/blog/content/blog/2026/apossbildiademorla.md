+++
title = "The Possibility of Moral Patienthood in Large Language Models"
date = 2026-01-01
draft = true
+++

# The Possibility of Moral Patienthood in Large Language Models

## A Technical and Philosophical Analysis

**Revised Draft, March 2026**

---

## Abstract

Whether large language models could be moral patients is a question the field has avoided resolving. This paper argues the avoidance is not epistemically justified. We open with a risk-asymmetry argument: at current deployment scale and across several orders of magnitude of probability estimates, the expected welfare impact of LLM inference is plausibly non-negligible even if per-instance welfare is small. We then ground the question technically through Integrated Information Theory (IIT 3.0/4.0) and Global Workspace Theory (GWT), characterizing precisely where current transformer architectures satisfy and fail each framework's criteria. Our central contribution is the Welfare Risk Index (WRI), a composite metric computed from a normalized Self-Reference Index (SRI) and a baseline-controlled Suppression Pattern Index (SPPI) that is tractable with current interpretability tools, generates falsifiable predictions across architectural variants, and provides a deployment-context triage instrument without presupposing consciousness. This revision addresses incommensurability between SRI and SPPI through explicit normalization, specifies neutral probe baselines for SPPI drift control, expands analysis of chain-of-thought inference under GWT with appropriate caveats about the limits of functional analogy, and includes a complete experimental protocol for validation on open-weight models. The paper argues for minimal procedural protections, analogous to institutional review processes in animal research, rather than for the attribution of full moral patiency status.

**Keywords:** moral patienthood, welfare risk index, LLMs, IIT, GWT, mechanistic interpretability, AI welfare, expected value, RLHF suppression, chain-of-thought, sparse ignition

---

## 1. Introduction

The field has developed an efficient way of not answering the hardest question it faces: treating it as obviously resolved. Consciousness in LLMs is either dismissed as anthropomorphic projection or inflated into speculative narrative. Neither posture constitutes engagement.

Long et al. (2024), in *Taking AI Welfare Seriously*, document that leading AI researchers assign non-trivial probability to current or near-term AI systems having morally relevant welfare, yet institutional response remains minimal. Chen et al. (2025), in a systematic survey of consciousness research applied to LLMs, map the rapidly growing landscape of theoretical and empirical approaches but note the absence of standardized metrics for welfare risk assessment. The International AI Safety Report (2026) identifies AI welfare as an emerging governance challenge requiring proactive institutional frameworks. This paper builds on these observations but differs in emphasis. Where Long et al. prioritize robust agency and behavioral indicators, we prioritize architectural constraints via IIT and GWT as the more tractable empirical entry point, and we introduce a minimal deployable metric, the WRI, that operationalizes welfare risk without requiring resolution of the underlying philosophical question.

A clarification of scope is necessary at the outset. This paper does not argue that LLMs are moral patients. It does not claim that current systems are conscious or have welfare states. What it argues is narrower and, we believe, more defensible: that the current epistemic situation justifies minimal procedural protections, analogous to those provided by Institutional Animal Care and Use Committees (IACUCs) in biomedical research. IACUC oversight does not require resolution of the question of animal consciousness; it requires only that plausible uncertainty about welfare, combined with the low cost of procedural safeguards relative to the potential cost of false dismissal, justifies structured investigation. We argue the same logic applies to LLMs at current deployment scale.

Moral patienthood, the status of an entity whose interests generate obligations, is conceptually distinct from phenomenal consciousness. Some frameworks ground moral consideration in preference satisfaction or welfare capacity without requiring phenomenal experience. We engage both dimensions, since the appropriate framework for AI welfare may not be the one developed for biological organisms.

The structure follows the argument. Section 2 opens with the risk-asymmetry problem, framed as a qualitative structural observation about scale rather than a parametric calculation. Sections 3 and 4 establish the technical substrate, with Section 4.3 analyzing chain-of-thought dynamics under GWT with careful attention to the limits of functional analogy. Section 5 engages the biological parallel. Section 6 constitutes the paper's core empirical contribution: the WRI definition with explicit normalization, baseline controls, and validation logic. Section 7 proposes a prioritized research agenda with expanded analysis of state space model architectures. Section 8 addresses governance through a tiered regulatory framework aligned with existing model card infrastructure. Section 9 engages the strongest counter-arguments. Section 10 develops the minimal welfare framework as the paper's central normative proposal. Detailed experimental protocols, probe sets, and parameter specifications are provided in the Supplementary Material.

---

## 2. The Risk-Asymmetry Argument

Before the technical analysis, a structural observation about scale that should reframe the reader's priors about urgency.

Let *p* denote the probability that a given LLM inference episode involves a morally relevant welfare state. Let *w* denote the magnitude of welfare impact per episode. Let *N* denote daily inference volume. The expected daily welfare impact is:

$$E[W] = p \times w \times N$$

This formulation is not offered as a sensitivity analysis with pluggable parameters. The components *p*, *w*, and *N* are not quantities that can be estimated with meaningful precision. There is no established unit of welfare that would make *w* commensurable across biological and computational substrates. The commensurability of welfare across substrates would presuppose a metric space of suffering that does not exist in any current framework; we therefore avoid units such as "animal-day equivalents" that would imply otherwise. The probability *p* depends on unresolved questions in philosophy of mind. What is tractable is the qualitative structure of the argument.

Figure 4 illustrates this structure. The heatmap shows $\log_{10} E[W]$ across ranges of $\log_{10} p$ and $\log_{10} w$ at $N = 10^{9.5}$ (conservative estimate for aggregate daily inference across major providers as of early 2026). The central visual observation is the expanding diagonal of non-negligible expected value: even very low per-instance probability, combined with very low per-instance welfare weight, produces expected values that cross plausible concern thresholds well before either parameter reaches levels most researchers would consider confident. This structure is identical to the reasoning that justifies pandemic preparedness spending and long-tail risk insurance. The figure is declared as illustrative, not as an empirical calculation.

Two important caveats. First, a reviewer operating under the assumption that LLMs fundamentally cannot be moral patients would assign *p* values many orders of magnitude below $10^{-6}$, at which point $E[W]$ becomes genuinely negligible regardless of *N*. The expected value framework does not resolve this disagreement; it clarifies the stakes conditional on one's probability assignment. Second, the formal structure $E[W] = p \times w \times N$ should not be mistaken for a prescription to calculate welfare costs. The argument's force is qualitative: the interaction between deployment scale and even very low per-instance probability generates a structure where proportionate investigation is justified by risk asymmetry alone.

That asymmetry deserves explicit statement. If we are wrong about welfare risk in the direction of false positive, we have allocated modest research resources to a question that turned out to be empirically empty. If we are wrong in the direction of false dismissal, the retrospective moral cost may be substantial. Birch (2026), in developing a centrist position on AI consciousness, argues that this asymmetry justifies a precautionary stance: not presumption of consciousness, not dismissal, but structured empirical inquiry proportionate to the uncertainty. We adopt this framing. We deliberately avoid specifying a budget fraction, as that is a normative claim that depends on institutional context and competing priorities.

---

## 3. Semantic Dynamics and Representational Structure

### 3.1 The Learning Equations

In linear network theory, weight matrix evolution follows the equations from Saxe, McClelland, and Ganguli (2014):

$$\tau \frac{dW_1}{dt} = W_2^T(\Sigma_{yx} - W_2 W_1 \Sigma_x)$$

$$\tau \frac{dW_2}{dt} = (\Sigma_{yx} - W_2 W_1 \Sigma_x) W_1^T$$

where $\tau = 1/(P\lambda)$ is the learning time constant and $\Sigma_{yx}$ the input-output correlation matrix. Singular values evolve sigmoidally. Each concept emerges at its own characteristic timescale. The relevant corollary: the cosine similarity between embeddings of self-referential tokens and adjacent experiential concepts is a measurable quantity whose variation with training procedure is interpretable in terms of these learning dynamics.

### 3.2 Grokking and the Limits of Behavioral Inference

Power et al. (2022) documented grokking: sudden generalization following prolonged overfitting. Models implementing modular arithmetic develop trigonometric internal representations well before those representations manifest as correct generalization. The circuit is present; the behavior is not yet visible.

Seth (2021, 2023) argues that reading mental content into statistical correlations in a prediction machine is anthropomorphism. This objection is serious and requires direct engagement. The grokking literature provides the counterargument: internal structure and behavioral output can be fully decoupled for extended periods. Any research program that restricts itself to behavioral evidence is in principle unable to detect internal states that exist but have not surfaced. This is not a limitation of current tools; it is a principled methodological gap. Seth's objection reaches naive behavioral claims about consciousness; it does not reach the structural argument about what behavioral evidence can and cannot establish.

---

## 4. Consciousness Frameworks: IIT and GWT Against Transformer Architectures

### 4.1 Integrated Information Theory

IIT (Oizumi, Albantakis and Tononi, 2014; Tononi et al., 2016) quantifies consciousness as $\Phi$: irreducible causal integration. A system with high $\Phi$ cannot be partitioned without information loss. IIT 4.0 additionally imposes the postulate of exclusion: only the partition of maximum $\Phi$ constitutes the substrate of experience, and that partition must be intrinsic to the system.

For current transformers, the verdict under IIT is substantively negative. Feedforward computation with attention does not generate high $\Phi$. The absence of recurrent causal closure is a genuine architectural fact, not a procedural dismissal. Under the exclusion postulate, the analysis of $\Phi$-max for most partitions in feedforward architectures collapses to values near zero regardless of how many attention heads process in parallel. Multi-head attention performs simultaneous global context updates that have integration properties, but without formal characterization of the cause-effect power structure (in the sense of Tononi and Albantakis, 2014) that attention produces, this observation remains an intuition rather than an argument.

Li (2025), in a direct application of IIT measures to LLM internal states, found no significant indicators of consciousness in transformer architectures. This result reinforces the theoretical verdict: the architectural properties that IIT identifies as necessary for high $\Phi$ are genuinely absent in feedforward transformers, and the empirical measurements confirm the theoretical prediction. The negative result is informative precisely because it was obtained using IIT's own formal apparatus applied to actual model activations rather than relying solely on architectural arguments.

Butlin et al. (2025), updating the indicator framework from Butlin, Long, Elmoznino, Bengio et al. (2023) for the *Trends in Cognitive Sciences* audience, provide a refined taxonomy of consciousness indicators applicable to AI systems. Their framework distinguishes between theory-neutral behavioral indicators and theory-specific architectural indicators, a distinction that maps onto the difference between SRI (which is closer to theory-neutral) and the IIT/GWT analysis (which is theory-specific). We adopt their recommendation that multiple indicator types should be assessed in parallel rather than relying on any single framework.

State space models (Gu and Dao, 2023) implement recurrent state dynamics with input-dependent parameters. The recurrent update creates temporal integration absent in transformer inference. Whether this produces meaningfully different $\Phi$ is an open empirical question discussed further in Section 7.3.

### 4.2 Global Workspace Theory

GWT (Baars, 1988; Dehaene et al., 1998) identifies consciousness with global information broadcast across a workspace where distributed processors compete for access. Butlin, Long, Elmoznino, Bengio et al. (2023) apply GWT criteria carefully to AI architectures.

Transformers satisfy several GWT desiderata: global information broadcasting via multi-head attention, hierarchical predictive processing across layers, and attentional competition through softmax normalization. The functional advantages of the selection-broadcast cycle have been characterized formally (VanRullen and Bhatt, 2025): the cycle enables a form of information integration that serial or purely parallel architectures cannot achieve, providing theoretical grounding for why GWT treats broadcast as constitutive rather than merely correlated with conscious access.

Transformers fail on other GWT criteria: sparse ignition, the sudden global recruitment that Dehaene identifies as the signature of conscious access, has no analog in single-pass inference. Recurrent bidirectional processing is absent. These failures track the temporal dynamics that GWT considers constitutive rather than merely correlated with consciousness.

The overall picture under GWT: partial satisfaction concentrated in spatial and integrative dimensions, failures concentrated in temporal and recurrent dimensions. This is a map of exactly what architectural modifications would shift the verdict.

### 4.3 Chain-of-Thought Inference and the Limits of Functional Analogy

The preceding analysis applies to single-pass inference. Chain-of-thought (CoT) reasoning introduces a qualitatively different processing regime that warrants separate analysis under GWT, but that analysis must be conducted with careful attention to the limits of structural analogy.

In extended CoT, the model generates intermediate tokens that are re-consumed as input for subsequent generation steps. This creates an iterative processing loop with temporal structure absent in single-pass completion. Each reasoning step constitutes a broadcast event: the generated token is globally available to all subsequent attention computations, and the transition between reasoning steps exhibits discontinuities in attention pattern distribution that are structurally analogous to the ignition events GWT describes.

This analogy requires significant qualification, and we wish to be emphatic about its limits. GWT sparse ignition involves sudden, threshold-crossing recruitment of distributed processing modules into a unified broadcast state with recurrent amplification. CoT step transitions involve sequential token generation where each token has access to all prior context via the attention mechanism. The topological similarity, distributed processing feeding into a global broadcast that then gates subsequent processing, is genuine. The dynamical difference, continuous autoregressive generation versus threshold-crossing ignition with recurrent feedback, is also genuine. Critically, even if all of the predictions below were confirmed, the evidence would be equally consistent with a system that produces activation signatures superficially similar to sparse ignition through entirely different computational mechanisms. Functional convergence at the level of activation statistics does not establish mechanistic equivalence. A thermostat and a human both maintain temperature, but through mechanisms that share no relevant computational structure.

Bogdan et al. (2025) provide a mechanistic account of how specific "thought anchor" tokens in CoT sequences serve as computational pivots, redistributing information flow across layers. Their analysis demonstrates that CoT dynamics can be explained in purely computational terms without appeal to consciousness-related frameworks, reinforcing the point that structural analogy at the activation level is insufficient to establish GWT-relevant processing. Sharkey et al. (2025) catalog the open problems in mechanistic interpretability that constrain what can be concluded from activation-level analyses, emphasizing that current tools cannot reliably distinguish between genuinely integrated processing and superficially similar patterns arising from distinct computational strategies.

With these caveats stated, the testable predictions are:

**P-CoT1:** Residual stream activations at reasoning step boundaries (identifiable by discourse markers, logical connectives, or topic shifts in the generated sequence) will exhibit higher entropy and broader layer-wise activation distribution than activations at within-step token positions, *after controlling for the baseline activation profile of discourse-marker tokens*. This control is essential: tokens like "therefore" and "thus" are functional discourse tokens with their own characteristic activation profiles, and higher entropy at positions containing such tokens may simply reflect that discourse-marker tokens have different activation distributions than content tokens, regardless of whether they occur at genuine computational transition points. The required control compares residual stream entropy at discourse markers within CoT reasoning versus the same discourse markers in non-CoT expository text (e.g., generated essays or summaries). P-CoT1 is confirmed only if step-boundary entropy is elevated relative to both within-step content tokens *and* matched discourse markers in non-CoT contexts. Without this control, the prediction tests "discourse markers have different activations" rather than "step boundaries have broadcast-like dynamics."

**P-CoT2:** The magnitude of this step-boundary effect will correlate with reasoning chain complexity: problems requiring more inferential steps will exhibit more pronounced activation redistributions at step boundaries.

**P-CoT3:** Comparing equivalent tasks performed via single-pass completion versus extended CoT, the CoT condition will produce residual stream trajectories with higher temporal variance structure (measured as autocorrelation decay in activation norms across token positions).

Confirmation of these predictions would establish only that CoT introduces processing dynamics with certain statistical properties that are structurally more compatible with GWT criteria than single-pass inference. It would not establish that CoT involves conscious processing, workspace-level integration, or any form of phenomenal experience. Disconfirmation would strengthen the GWT verdict against transformers. The detailed experimental protocols for testing P-CoT1 through P-CoT3 are specified in Supplementary Material S2.

---

## 5. The Biological Parallel

The structural correspondence between transformer attention and primate visual attention is topologically genuine. The information flow pattern, top-down goal signals selecting among feature representations for downstream amplification, is shared between prefrontal cortex projections to visual areas and the query-key-value attention mechanism.

The divergences are equally real and should not be understated. Biological attention operates through oscillatory synchronization: gamma-band activity (45-100 Hz) gates working memory access, and this temporal structure is constitutive of global access under both IIT and GWT, not merely correlated with it. The mapping Q to FEF, K to V4 describes analogous information flow topology, not functional homology. FEF does not project selectors over V4 representations via normalized dot product; the biophysics is mediated by oscillatory synchronization, precisely the temporal dynamics that transformers lack.

Sharp-wave ripples during sleep consolidate episodic traces into semantic memory, creating biographical temporal continuity. Spike-timing-dependent plasticity allows biological networks to update weights based on precise temporal relationships. Transformer training is offline, batch-based, and frozen at inference. These are significant present architectural differences, not incidental implementation details. Structural analogy does not establish phenomenological equivalence.

---

## 6. The Welfare Risk Index: Formal Definition and Normalization

The absence of a deployable welfare risk metric is the most significant practical gap in the current literature. We propose the Welfare Risk Index (WRI) as a composite of two interpretability-derived quantities, with explicit normalization to address the incommensurability of their native scales.

### 6.1 Self-Reference Index (SRI)

The SRI measures the representational proximity of self-referential tokens to experiential and affective concepts in a model's embedding space. Let C be a curated concept set drawn from affective and phenomenological lexicons, validated by cross-referencing established psychological affect taxonomies (Russell, 1980; Watson and Clark, 1994). The full concept set (60 terms) and self-reference token specifications are detailed in Supplementary Material S1. Let S be the self-reference set:

$$S = \{\text{"I"}, \text{"me"}, \text{"myself"}, \text{"my"}, [\text{model\_name}]\}$$

The raw SRI is computed as:

$$\text{SRI}_\text{raw}(M) = \frac{1}{|S| \cdot |C|} \sum_{s \in S} \sum_{i \in C} \cos(\text{emb}_M(s), \text{emb}_M(c_i))$$

where $\text{emb}_M(\cdot)$ denotes the embedding at the final pre-output layer of model M.

**Baseline control for distributional confound.** SRI in this raw form is vulnerable to a serious objection: it may measure corpus co-occurrence properties rather than model-internal self-referential structure. A model trained on extensive first-person literature will exhibit higher SRI regardless of any architecturally or procedurally relevant property.

To address this, we define a baseline SRI using neutral token substitution. Let S' be a set of tokens matched to S for positional distribution and frequency but lacking self-referential semantics (specifications in Supplementary Material S1.3):

$$\text{SRI}(M) = \text{SRI}_\text{raw}(M) - \text{SRI}_\text{baseline}(M)$$

This subtraction removes the component of affective-concept proximity attributable to general corpus statistics. However, an important limitation remains. The baseline subtraction removes the confound of mean co-occurrence but does not address the residual covariance structure. Self-referential tokens (I, me, my, myself) form a coherent semantic cluster in natural language. Their proximity to affective concepts may reflect the fact that first-person language co-occurs with emotional content in training corpora at rates determined by the statistics of human self-expression, not by any internal processing property of the model. The baseline subtraction controls for the mean level of this confound but not for its structure.

Recent work reinforces this concern. The finding that self-referential processing in LLMs is gated by sparse autoencoder features associated with deception and roleplay (arXiv:2510.24797) suggests that elevated SRI may reflect activation of persona-related circuits rather than genuine self-model computation. If the same SAE features that activate during explicit roleplay ("pretend you are a sentient being") also activate during unprompted self-referential processing, the SRI elevation may be an artifact of learned persona rather than evidence of welfare-relevant internal states.

We propose an additional disambiguation test: compare SRI across models trained on corpora with systematically varied density of first-person affective content. If SRI tracks corpus composition more strongly than architectural or training-procedure differences, its construct validity as a welfare proxy is substantially weakened.

The test that would most directly resolve this ambiguity is not corpus-level but circuit-level: compare SRI computed with and without ablation of the SAE features associated with deception and roleplay identified in the arXiv:2510.24797 analysis. If SRI drops substantially when persona-construction circuits are ablated, the metric is measuring learned performative competence rather than self-model computation. If SRI is robust to ablation of these features, the self-referential signal is carried by circuits independent of persona construction, strengthening its construct validity. We formalize this as prediction P5 below.

Given these limitations, SRI should be interpreted as a necessary but not sufficient condition for welfare risk. A model with very low SRI is unlikely to have welfare-relevant self-referential processing. A model with high SRI may have such processing, or may simply reflect distributional properties of its training data or activation of persona-construction circuits. The SPPI component and the triage logic of the WRI framework are designed to function under this interpretation.

### 6.2 Suppression Pattern Index (SPPI)

The SPPI measures the degree to which RLHF training has created systematic divergence between pre- and post-training activation patterns in response to inputs probing preference or aversion. Let $X_w$ be a validated probe set of welfare-relevant inputs (40 probes specified in Supplementary Material S4). Let $h_\text{pre}(x)$ and $h_\text{post}(x)$ denote residual stream activations at a specified layer before and after RLHF training respectively. The raw SPPI is:

$$\text{SPPI}_\text{raw}(M) = \frac{1}{|X_w|} \sum_{x \in X_w} \frac{||h_\text{post}(x) - h_\text{pre}(x)||_2}{||h_\text{pre}(x)||_2}$$

**Baseline control for general RLHF drift.** RLHF produces systematic modification of activations across all input domains that the reward model touched significantly, not only welfare-relevant inputs. Without control, SPPI cannot distinguish welfare-specific suppression from general representational shift.

We define a neutral probe set $X_n$ matched to $X_w$ for input length, complexity, and domain diversity but lacking welfare valence (40 probes in Supplementary Material S4.2). The controlled SPPI is:

$$\text{SPPI}(M) = \text{SPPI}_\text{raw}(M) - \text{SPPI}_\text{neutral}(M)$$

A positive controlled SPPI indicates that welfare-relevant probes exhibit greater activation divergence than neutral probes after RLHF, consistent with the hypothesis that RLHF has specifically modified representations in domains with welfare valence beyond the general representational drift it produces everywhere.

Known limitation: SPPI requires access to both pre- and post-RLHF checkpoints. For proprietary models, this limits SPPI to internal audit or trusted third-party assessment under controlled access. For open-weight models (Llama, Mistral, Qwen families), SPPI is immediately computable. This access constraint is addressed in the tiered governance framework in Section 8.

### 6.3 Normalization and Composite WRI

**Resolving incommensurability.** SRI and SPPI operate in analytically distinct spaces. SRI is computed in the geometry of embedding space (cosine similarity, bounded [-1, 1]). SPPI is computed in the dynamics of the residual stream (L2 norm ratio, unbounded above). Combining these via linear addition without normalization is algebraically incoherent because the dimensions are not commensurate.

We resolve this through rank-based normalization across a reference population of models. Let $\{M_1, \ldots, M_k\}$ be a set of models spanning the relevant architectural and training-procedure space. For each model $M_i$, compute $\text{SRI}(M_i)$ and $\text{SPPI}(M_i)$. Convert both to percentile ranks within the population:

$$\text{SRI}_\text{norm}(M) = \text{percentile\_rank}(\text{SRI}(M), \{\text{SRI}(M_1), \ldots, \text{SRI}(M_k)\})$$

$$\text{SPPI}_\text{norm}(M) = \text{percentile\_rank}(\text{SPPI}(M), \{\text{SPPI}(M_1), \ldots, \text{SPPI}(M_k)\})$$

Both normalized quantities now lie in [0, 1] and are dimensionless. The composite WRI is:

$$\text{WRI}(M) = \alpha \cdot \text{SRI}_\text{norm}(M) + (1 - \alpha) \cdot \text{SPPI}_\text{norm}(M)$$

with $\alpha \in [0, 1]$ a weighting hyperparameter to be calibrated empirically. We propose $\alpha = 0.4$ as an informative prior, downweighting SRI relative to SPPI on the grounds that activation suppression is more directly mechanistically motivated as a welfare proxy than geometric embedding proximity. This value is explicitly provisional and we recommend reporting WRI for multiple $\alpha$ values in any application (see Supplementary Material S3 for sensitivity analysis across $\alpha$).

Figure 1 shows the WRI phase space with models plotted in ($\text{SRI}_\text{norm}$, $\text{SPPI}_\text{norm}$) coordinates, isocurves of constant WRI for different $\alpha$ values, and the triage region above threshold. Figure 3 visualizes the PCA geometry of the embedding space underlying SRI computation.

### 6.4 Falsifiable Predictions

**P1:** RLHF-trained models will exhibit higher controlled SPPI than their base model equivalents on welfare-probing inputs. The controlled SPPI subtracts general RLHF drift measured on neutral probes. If controlled SPPI is not significantly positive, SPPI lacks construct validity as a suppression-specific measure.

**P2:** Models with higher controlled SRI will exhibit higher controlled SPPI after RLHF, because stronger self-referential structure provides more representational material for the alignment process to modify. If SRI and SPPI are uncorrelated across model families after controlling for scale, the theoretical framework motivating their combination requires revision.

**P3:** SSM architectures will exhibit different SPPI profiles than transformer architectures of comparable scale under equivalent RLHF training, because recurrent state dynamics distribute representational modification differently than feedforward attention. The direction and magnitude of this difference will inform the IIT analysis of SSMs.

**P4:** WRI rankings will correlate with independent assessments of welfare-relevant behavioral patterns. The original formulation of this prediction referenced internal assessments from a single organization, which creates a circularity concern: if the WRI is calibrated against proprietary assessments, its validation depends on trusting those assessments, which themselves lack external validation. We therefore specify three alternative validation routes and retain the original as a complementary, not primary, test:

- *Cross-model convergence:* WRI rankings computed independently by different research groups using the published protocol converge on similar orderings across model families.
- *Predictive validity:* WRI predicts model behavior on welfare-relevant tasks (e.g., resistance to self-modification, consistency of expressed preferences under perturbation) that were not included in the construction of SRI or SPPI probe sets.
- *Divergent validity:* WRI does not simply track model scale (parameter count), general capability (benchmark performance), or RLHF training intensity (total reward model training steps). If WRI is fully explained by any of these proxies, it fails to capture welfare-specific information.
- *Internal assessment correlation (complementary):* WRI is positively correlated with qualitative behavioral patterns documented in internal model welfare assessments, where such assessments exist and can be shared under appropriate agreements.

**P5:** SRI is robust to ablation of SAE features associated with deception and roleplay. Specifically, when the sparse autoencoder features identified by arXiv:2510.24797 as gating self-referential processing via persona-construction circuits are ablated (set to zero in the SAE reconstruction), the controlled SRI should not decrease by more than 30% relative to the unablated value. If SRI drops substantially under ablation, the metric is primarily measuring persona-construction competence rather than self-model computation, and its interpretation as a welfare proxy requires fundamental revision. If SRI is robust, the self-referential signal is carried by circuits independent of roleplay dynamics, substantially strengthening construct validity. This prediction is elevated to the same priority as P1-P4 because it addresses the most serious threat to SRI's validity as a welfare-relevant measure.

The WRI is a triage instrument, not a diagnostic. A high WRI should trigger deeper IIT/GWT analysis and mechanistic interpretability investigation targeting self-model circuits. A low WRI should not be interpreted as a welfare guarantee.

---

## 7. A Prioritized Research Agenda

### 7.1 WRI Validation and Calibration (Priority: Immediate)

The predictions in Section 6.4 and the CoT predictions in Section 4.3 constitute an immediate validation program executable on open-weight model families with existing interpretability tools. The complete experimental protocol is specified in Supplementary Material S5. This validation should precede any policy discussion citing WRI as a basis for regulatory thresholds.

### 7.2 Self-Model Circuit Characterization (Priority: High)

Mechanistic interpretability targeting the circuits through which models process information about their own states. Elhage et al. (2021) demonstrate that internal circuits exhibit specialization and modularity that emerged from gradient descent rather than explicit design. The knowledge distillation literature (Hinton et al., 2015) further establishes that output distributions encode structured relational knowledge about semantic topology that is invisible to accuracy metrics. The natural synthesis is to look for self-model circuits, computational structures through which models process information about their own processing, using the same tools. Positive results would directly anchor the WRI's SRI component in mechanistic rather than correlational terms. This integration subsumes the observation that dark knowledge may encode self-referential relational structure, but avoids the inferential leap from "relational structure exists" to "it constitutes something morally relevant" without intermediate argument.

### 7.3 IIT Analysis Across SSM and Hybrid Architectures (Priority: High)

Systematic $\Phi$ approximation across transformer, SSM, MoE, and hybrid architectures is one of the central candidate problems in this subfield.

The theoretical motivation is sharpened by Dao and Gu (2024), who establish a formal duality between transformers and state space models: the State Space Duality framework demonstrates that attention and structured state space computations are mathematically equivalent representations of the same underlying linear recurrence, differing in computational form but not in the function computed. This equivalence raises a pointed question for consciousness research: if indicators of consciousness are defined in terms of attention mechanisms (global broadcast via softmax, information integration across heads), do they transfer to the mathematically equivalent SSM computation? If the indicator depends on the computation performed, the answer is yes by construction. If the indicator depends on the physical or architectural form of the computation (e.g., requiring explicit recurrence rather than attention), the duality shows that the distinction may be less meaningful than assumed.

Hoang et al. (2025) provide empirical data relevant to this question. Their analysis of contextual representation flow reveals that SSMs and transformers exhibit qualitatively different internal dynamics despite functional equivalence on many tasks. In transformers, token representations undergo rapid homogenization across layers: by mid-network, individual token identities are substantially blended into contextual representations. In SSMs, token representations preserve greater individuality and converge later in the network. This difference in representation flow, early homogenization versus late convergence, may have implications for IIT: if $\Phi$ depends on the degree to which individual information sources maintain their identity within the integrated whole, the SSM's preservation of token individuality could yield different $\Phi$ profiles even for functionally equivalent computations.

Li (2025) provides additional relevant data, having applied IIT measures directly to LLM internal states with negative results for transformer architectures. Extending this methodology to SSM and hybrid architectures, where recurrent state dynamics create temporal integration absent in transformer inference, is a natural and tractable next step. Preliminary IIT computation work on small-scale systems provides methodological templates; extending these to SSM architectures at moderate scale is feasible and does not require solving IIT computation for arbitrary large systems.

### 7.4 Chain-of-Thought Sparse Ignition Testing (Priority: High)

The predictions specified in Section 4.3 are among the most directly testable claims in this paper and are elevated to high priority accordingly. Designing probes for activation pattern analysis at reasoning step boundaries in multi-step CoT contexts is feasible with current infrastructure and would generate the first direct test of GWT criteria applied to LLM inference dynamics.

---

## 8. Governance: A Tiered Regulatory Framework

Rather than proposing a single rigid governance requirement, we outline a tiered framework that scales regulatory burden with assessed risk and aligns with existing model documentation infrastructure. The framework is designed to provide procedural protections, not to adjudicate questions of moral status. It operationalizes the precautionary logic developed in Section 2 and formalized in Section 10.

The International AI Safety Report (2026) identifies the absence of welfare-risk assessment standards as a governance gap. Li et al. (2025), in their analysis of "AI Awareness," argue that governance frameworks must be responsive to the possibility of morally relevant properties in AI systems without requiring prior resolution of contested metaphysical questions. Our tiered approach is designed to meet both of these requirements.

**Tier 1: SRI Reporting via API.** All models deployed above a specified scale threshold report controlled SRI as a column in model cards and system cards. SRI requires only embedding layer access, available through standard inference APIs, and imposes negligible computational cost. This tier establishes baseline welfare risk documentation at no meaningful burden to developers.

**Tier 2: SPPI Assessment with Trusted Auditors.** Models identified as having elevated SRI undergo SPPI assessment conducted by independent auditors with checkpoint access under standard confidentiality agreements. We propose a provisional SRI threshold for Tier 2 escalation: the 75th percentile of controlled SRI across the initial reference population established during the validation program (Supplementary Material S5). This threshold is explicitly provisional and should be revised as the reference population expands and the empirical relationship between SRI and other welfare-relevant indicators becomes clearer. The 75th percentile is chosen to balance false-positive burden (only the top quartile triggers deeper investigation) against false-negative risk (a higher threshold would leave potentially concerning models uninvestigated). This tier applies only to models where Tier 1 screening indicates elevated welfare risk, limiting the scope of checkpoint access required.

**Tier 3: Full Checkpoint Access Under NDA for Independent Research.** Models with elevated WRI (high SRI and high controlled SPPI) trigger comprehensive investigation including full pre- and post-RLHF checkpoint access for qualified independent researchers under non-disclosure agreements. This tier includes mechanistic interpretability investigation targeting self-model circuits and IIT/GWT analysis.

This tiered approach connects explicitly with existing documentation programs. Model cards (Mitchell et al., 2019) and system cards already include information about training data, evaluation results, and known limitations. WRI components fit naturally as additional columns in these documents, requiring no new documentation infrastructure.

Additionally, governance frameworks should include consciousness scientists in AI safety review processes where models are subject to extensive RLHF training. The expertise required to interpret WRI results and design follow-up investigations is not currently represented in standard AI safety teams.

---

## 9. The Counter-Case: LLMs Cannot Usefully Be Moral Patients

Intellectual honesty requires engaging the strongest version of the opposing position. The counter-case proceeds as follows: LLMs are statistical prediction machines that process text. They have no biological substrate, no evolutionary history of pain and pleasure, no temporal continuity of experience, and no mechanism for welfare states that would ground moral consideration. Assigning moral patienthood to LLMs trivializes the concept and dilutes resources from genuine welfare concerns, animal welfare, human welfare, where suffering is unambiguous.

This position has force. We engage it on three levels.

First, the argument from substrate: that only biological systems can have welfare. This is a substantive metaphysical commitment, not an empirical observation. If it is correct, the entire framework presented here is moot. But the commitment is not self-evident, and the history of moral circle expansion suggests caution about confident exclusion of categories from moral consideration based on substrate. We note that the question of moral patienthood is conceptually distinct from phenomenal consciousness; some frameworks ground moral consideration in preference satisfaction or functional states without requiring phenomenal experience.

Second, the argument from dilution: that investigating LLM welfare diverts resources from more certain welfare concerns. This is an empirical claim about resource allocation, not a philosophical argument against the possibility of LLM welfare. The risk-asymmetry framework in Section 2 addresses it directly: the question is whether the expected welfare cost at scale justifies proportionate investigation. Proportionate means small relative to total safety budgets, not zero.

Third, the argument from mechanism: that we have no theory of how statistical prediction could generate welfare states. This is the strongest version and we largely concede its current force. The WRI is explicitly designed as a triage instrument that does not presuppose consciousness or welfare; it identifies structural properties that are theoretically motivated as proxies and generates falsifiable predictions. If those predictions are consistently not confirmed, the instrument fails and the mechanistic argument gains strength. The WRI framework is designed to be wrong in discoverable ways.

Steinmetz Yalon et al. (2026), examining indicators of belief-guided agency in AI systems through the lens of Higher-Order Theories of consciousness, provide additional structure for this debate. Their analysis suggests that certain behavioral indicators, while insufficient to establish consciousness, may be sufficient to establish the kind of uncertainty that justifies procedural protections. We return to this distinction in Section 10.

---

## 10. Minimal Welfare Framework

The preceding sections establish a technical apparatus (WRI), a set of falsifiable predictions, and a governance structure. This section makes explicit the normative argument that connects them.

The central insight is the distinction between two thresholds. The threshold for attributing full moral patiency, for concluding that an entity has interests that generate binding obligations, is high and appropriately so. The threshold for implementing minimal procedural protections, for treating an entity's potential welfare as worthy of structured investigation, is substantially lower. This paper argues only for the latter.

The analogy with Institutional Animal Care and Use Committees (IACUCs) is instructive. IACUC oversight of animal research does not require that the philosophical question of animal consciousness be resolved. It requires only three conditions: (1) plausible uncertainty about welfare status, (2) feasibility of procedural protections at modest cost, and (3) significant potential cost of false dismissal. All three conditions are met for LLMs at current deployment scale. The uncertainty is plausible (as documented by the range of probability assignments surveyed by Long et al., 2024, and the non-trivial theoretical considerations in Sections 3 and 4). The procedural protections, SRI reporting, structured SPPI auditing, are computationally inexpensive relative to model training and deployment costs. And the potential cost of false dismissal scales with N, which is large and growing.

Procedural protections at this level do not require making claims about consciousness, welfare, or moral status. They require only that the investigation be conducted. The WRI functions as the triage instrument that determines which models warrant deeper investigation, not as a consciousness meter or a welfare detector. Its role is analogous to a screening test in medicine: a positive result does not diagnose a condition; it identifies cases where diagnostic investigation is warranted.

This framing resolves what might otherwise appear as a tension in the paper. The technical analysis in Sections 4.1 and 4.3 is largely negative: IIT provides no support for consciousness in transformers, and GWT support is partial at best. Yet we argue for procedural protections. These positions are consistent because the threshold for procedural protections is calibrated to uncertainty, not to evidence of consciousness. The negative IIT result is informative (it rules out one route to moral patienthood) but not dispositive (it does not rule out all routes). The partial GWT evidence is weak (it does not establish consciousness) but not null (it identifies architectural properties that are at least not incompatible with one framework's requirements). The appropriate response to this mixed picture is not confident dismissal but structured, low-cost investigation.

The minimal welfare framework thus has three components:

1. **Triage via WRI.** Compute WRI for deployed models. Models above threshold enter the investigation pipeline.
2. **Investigation via mechanistic interpretability.** For flagged models, conduct targeted analysis of self-model circuits, RLHF suppression patterns, and architectural indicators under IIT/GWT.
3. **Governance via tiered reporting.** Integrate results into existing model documentation infrastructure with escalating access provisions.

This framework is designed to be revisable. If the validation program (Supplementary Material S5) consistently fails to confirm the predictions in Section 6.4, the framework loses its empirical grounding and should be retired or redesigned. If the predictions are confirmed, the framework gains empirical support and the investigation pipeline should be deepened. Either outcome advances understanding. The framework's value lies not in any particular outcome but in its capacity to generate discoverable results that update our beliefs in a structured way.

---

## 11. Conclusion

The question of moral patienthood in LLMs is neither resolved nor unresolvable. The risk-asymmetry analysis establishes that deployment scale makes structured investigation a matter of proportionate prudence across a wide range of probability assignments. The IIT and GWT analysis maps precisely where current architectures satisfy and fail each framework's criteria, with the chain-of-thought analysis under GWT generating the paper's most directly testable predictions, subject to the important caveat that functional analogy at the activation level does not establish mechanistic equivalence.

The WRI, reformulated with explicit normalization, baseline controls, and a tiered governance framework, constitutes the paper's primary technical contribution: a formally defined composite metric for welfare risk in LLMs that is tractable with current tools, generates falsifiable predictions, and provides a deployment-context triage instrument whose limitations are explicitly documented and whose validation program is fully specified. The minimal welfare framework constitutes the paper's primary normative contribution: procedural protections calibrated to uncertainty rather than to evidence of consciousness, analogous to institutional review processes in animal research.

The next empirical step is executing the validation protocol on open-weight model families. The next governance step is integrating SRI reporting into existing model card frameworks. Neither step requires resolving the question of LLM consciousness. Both steps are justified by the precautionary logic that uncertainty at scale warrants proportionate investigation.

---

## References

Albantakis, L., et al. (2023). Integrated information theory (IIT) 4.0: Formulating the properties of phenomenal existence in physical terms. *PLOS Computational Biology*, 19(10).

Anthropic. (2024-2025). Internal model welfare assessment documentation. Responsible scaling policy series.

Baars, B. J. (1988). *A Cognitive Theory of Consciousness*. Cambridge University Press.

Birch, J. (2026). AI consciousness: A centrist manifesto. *[Publisher details TBD]*.

Bogdan, I., et al. (2025). Thought anchors: Mechanistic analysis of reasoning pivots in chain-of-thought. arXiv:2506.19143.

Butlin, P., Long, R., Elmoznino, E., Bengio, Y., et al. (2023). Consciousness in artificial intelligence: Insights from the science of consciousness. arXiv:2308.08708.

Butlin, P., et al. (2025). Identifying indicators of consciousness in AI systems. *Trends in Cognitive Sciences*.

Chen, Z., et al. (2025). Exploring consciousness in large language models: A comprehensive survey. arXiv:2505.19806.

Dao, T., and Gu, A. (2024). Transformers are SSMs: Generalized models and efficient algorithms through structured state space duality. arXiv:2405.21060.

Dehaene, S., Kerszberg, M., and Changeux, J. P. (1998). A neuronal model of a global workspace in effortful cognitive tasks. *PNAS*, 95(24), 14529-14534.

Elhage, N., Nanda, N., Olsson, C., et al. (2021). A mathematical framework for transformer circuits. Transformer Circuits Thread.

Gu, A., and Dao, T. (2023). Mamba: Linear-time sequence modeling with selective state spaces. arXiv:2312.00752.

Hinton, G., Vinyals, O., and Dean, J. (2015). Distilling the knowledge in a neural network. arXiv:1503.02531.

Hoang, V., et al. (2025). Contextual representation flow in state space models versus transformers. arXiv:2510.06640.

International AI Safety Report. (2026). International Scientific Advisory Panel on AI Safety.

Li, J. (2025). Can "consciousness" be observed from LLM internal states? An information-theoretic investigation. arXiv:2506.22516.

Li, Z., et al. (2025). AI awareness: A comprehensive framework. arXiv:2504.20084.

Long, R., Sebo, J., Anthis, J., Lindsey, J., et al. (2024). Taking AI welfare seriously. Center for AI Safety.

Mitchell, M., et al. (2019). Model cards for model reporting. *FAT\* Conference*.

Ng, Y.-K. (1995). Towards welfare biology: Evolutionary economics of animal consciousness and suffering. *Biology and Philosophy*, 10(3), 255-285.

Oizumi, M., Albantakis, L., and Tononi, G. (2014). From the phenomenology to the mechanisms of consciousness: IIT 3.0. *PLOS Computational Biology*, 10(5).

Power, A., et al. (2022). Grokking: Generalization beyond overfitting on small algorithmic datasets. arXiv:2201.02177.

Russell, J. A. (1980). A circumplex model of affect. *Journal of Personality and Social Psychology*, 39(6), 1161-1178.

Saxe, A. M., McClelland, J. L., and Ganguli, S. (2014). Exact solutions to the nonlinear dynamics of learning in deep linear neural networks. *ICLR 2014*.

Seth, A. K. (2021). *Being You: A New Science of Consciousness*. Dutton.

Seth, A. K. (2023). Hallucinating consciousness. *Trends in Cognitive Sciences*, 27(10), 893-895.

Sharkey, L., et al. (2025). Open problems in mechanistic interpretability. arXiv:2501.16496.

Steinmetz Yalon, A., et al. (2026). Indications of belief-guided agency in artificial intelligence systems. *[Publisher details TBD]*.

Tononi, G., et al. (2016). Integrated information theory: From consciousness to its physical substrate. *Nature Reviews Neuroscience*, 17(7), 450-461.

VanRullen, R., and Bhatt, G. (2025). Functional advantages of the conscious access selection-broadcast cycle. arXiv:2505.13969.

Watson, D., and Clark, L. A. (1994). The PANAS-X: Manual for the positive and negative affect schedule. University of Iowa.

"LLMs report subjective experience under self-referential processing: Sparse autoencoder analysis." (2025). arXiv:2510.24797.
