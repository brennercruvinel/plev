# Improvements

A log of the SEO, performance, PWA and content work done on this theme, why each
change was made, and links to the originating feature request and the pull
request that resolved it.

Repository: <https://github.com/whoisbrenner/brennercruvinel.com>

The work follows the 2026 search reality: ranking a list and *being cited* by
generative engines are two different games. The base game (clean content,
semantic HTML, real authority) is reinforced with a machine-readable layer
(structured data, entity graph, answer-up-front content, explicit crawler
policy) so the same site competes in both.

## Summary

| # | Area | Change | Issue | PR |
|---|------|--------|-------|----|
| 1 | SEO | hreflang alternates in `<head>` + `x-default` | [#1](https://github.com/whoisbrenner/brennercruvinel.com/issues/1) | [#2](https://github.com/whoisbrenner/brennercruvinel.com/pull/2) |
| 2 | SEO | Complete Open Graph + Twitter Card metadata | [#3](https://github.com/whoisbrenner/brennercruvinel.com/issues/3) | [#4](https://github.com/whoisbrenner/brennercruvinel.com/pull/4) |
| 3 | SEO | JSON-LD structured data as a single `@graph` | [#5](https://github.com/whoisbrenner/brennercruvinel.com/issues/5) | [#6](https://github.com/whoisbrenner/brennercruvinel.com/pull/6) |
| 4 | SEO | `BreadcrumbList` structured data + visible breadcrumb | [#7](https://github.com/whoisbrenner/brennercruvinel.com/issues/7) | [#8](https://github.com/whoisbrenner/brennercruvinel.com/pull/8) |
| 5 | SEO/AEO | AI/AEO metadata, TL;DR block, Speakable, AI-crawler `robots.txt` | [#9](https://github.com/whoisbrenner/brennercruvinel.com/issues/9) | [#10](https://github.com/whoisbrenner/brennercruvinel.com/pull/10) |
| 6 | Performance | Preload primary font + self-hosted bundled fonts | [#11](https://github.com/whoisbrenner/brennercruvinel.com/issues/11) | [#12](https://github.com/whoisbrenner/brennercruvinel.com/pull/12) |
| 7 | Performance | Automatic image `width`/`height` at build (anti-CLS) | [#13](https://github.com/whoisbrenner/brennercruvinel.com/issues/13) | [#14](https://github.com/whoisbrenner/brennercruvinel.com/pull/14) |
| 8 | PWA | Web app manifest, install icons, offline service worker | [#15](https://github.com/whoisbrenner/brennercruvinel.com/issues/15) | [#16](https://github.com/whoisbrenner/brennercruvinel.com/pull/16) |
| 9 | Content | Seed demo post archive (2009–2026) | [#17](https://github.com/whoisbrenner/brennercruvinel.com/issues/17) | [#18](https://github.com/whoisbrenner/brennercruvinel.com/pull/18) |

---

## 1. hreflang alternates + `x-default`

**Issue [#1](https://github.com/whoisbrenner/brennercruvinel.com/issues/1) · PR [#2](https://github.com/whoisbrenner/brennercruvinel.com/pull/2)**

Emit `<link rel="alternate" hreflang>` for the current page/section and every
translation, plus an `x-default` pointing at the default language.

**Why.** The theme supports many languages and has a UI language switcher, but it
declared no alternates to crawlers. Without hreflang, search engines may treat
translations as duplicate content and surface the wrong-language result.

**Files.** `templates/partials/alternates.html` (included in `head.html` after
the canonical link).

## 2. Complete Open Graph + Twitter Card metadata

**Issue [#3](https://github.com/whoisbrenner/brennercruvinel.com/issues/3) · PR [#4](https://github.com/whoisbrenner/brennercruvinel.com/pull/4)**

Add the missing social properties: `og:type` (article for posts, website
otherwise), `og:image` width/height/alt, `article:published_time` /
`modified_time` / `author` / `tag`, and the full Twitter Card set
(`summary_large_image`, title, description, image, site, creator).

**Why.** Links shared to X showed no rich card, and Facebook/LinkedIn lacked
`og:type` and article metadata, producing weak previews. OG images stay on the
native Zola pipeline (`resize_image` / static card) — no Node dependency.

**Files.** `templates/partials/social-meta.html`; `config.extra.twitter_username`.

## 3. JSON-LD structured data as a single `@graph`

**Issue [#5](https://github.com/whoisbrenner/brennercruvinel.com/issues/5) · PR [#6](https://github.com/whoisbrenner/brennercruvinel.com/pull/6)**

Emit one `@graph` per page instead of disconnected JSON-LD blocks. Persistent
entities are anchored at the domain root with stable `@id` and referenced
everywhere by `@id`:

- `Organization` at `/#organization` with `sameAs` (derived from
  `footer.socials`, plus optional extra profiles such as Wikidata) and
  `knowsAbout`.
- `WebSite` at `/#website` (publisher references the organization), optional
  `SearchAction`.
- `Person` at `/#person` for the author, optional `sameAs`.
- Page node anchored at the URL: `BlogPosting` (`url#article`) for posts,
  `WebPage` (`url#webpage`) otherwise.

**Why.** Engines reason about *entities*, not pages. A single graph with stable
`@id` lets the brand resolve as one node and connect to the knowledge graph;
`sameAs` borrows authority from third-party profiles. Centralizing generation in
one partial avoids the plugin-style duplication (two diverging `Organization`
blocks) that breaks entity resolution.

**Files.** `templates/partials/schema.html`; entity config keys in `config.toml`
(`organization_knows_about`, `organization_sameas`, `author_sameas`,
`search_url`).

## 4. `BreadcrumbList` + visible breadcrumb

**Issue [#7](https://github.com/whoisbrenner/brennercruvinel.com/issues/7) · PR [#8](https://github.com/whoisbrenner/brennercruvinel.com/pull/8)**

Add a `BreadcrumbList` node to the same `@graph` (built from `page.ancestors`)
and a visible breadcrumb nav on pages and posts that mirrors the same trail.
Skipped on the homepage.

**Why.** There was no hierarchy signal and no trail for users. The visible HTML
and the structured data are kept in agreement (triple concordance: visible
text == semantic HTML == JSON-LD), which is what engines reward and what avoids
spam signals.

**Files.** `templates/partials/breadcrumbs.html`, breadcrumb node in
`schema.html`, `sass/_breadcrumb.scss`; included in `article.html` and
`page.html`.

## 5. AI/AEO metadata, TL;DR block, Speakable, AI-crawler `robots.txt`

**Issue [#9](https://github.com/whoisbrenner/brennercruvinel.com/issues/9) · PR [#10](https://github.com/whoisbrenner/brennercruvinel.com/pull/10)**

Answer-engine optimization bundle:

- `ai.*` meta hints (content type, language, publisher, indexable, summary,
  word count, reading time, related topics, topics).
- Optional TL;DR block at the top of a post (from `page.extra.tldr`), visible,
  carrying `data-ai-summary` — the answer up front, in the first paragraphs.
- `SpeakableSpecification` on the `BlogPosting` node (`.tldr`, `h1`, `h2`).
- `templates/robots.txt` that explicitly allows the AI/LLM crawlers (GPTBot,
  OAI-SearchBot, PerplexityBot, ClaudeBot, Google-Extended, CCBot, …) and
  declares the sitemap.

**Why.** Generative engines cite content that is easy to extract: an answer up
front, clear structure, and an explicit crawl policy. The site previously gave
them no extraction hints and no crawler policy, making it hard to cite.

**Files.** `templates/partials/ai-meta.html`, `templates/robots.txt`,
TL;DR block in `article.html`, `sass/_tldr.scss`, Speakable in `schema.html`.

## 6. Font preload + self-hosted bundled fonts

**Issue [#11](https://github.com/whoisbrenner/brennercruvinel.com/issues/11) · PR [#12](https://github.com/whoisbrenner/brennercruvinel.com/pull/12)**

Preload `InterVariable.woff2` (`as=font`, `crossorigin`) when bundled fonts are
enabled, and turn `bundled_fonts` on by default. `font-display: swap` was already
set on every `@font-face`.

**Why.** Without preload the primary font is discovered late (after CSS),
delaying LCP. Self-hosting removes third-party requests and improves privacy and
loading.

**Files.** `templates/partials/head.html`, `config.toml`.

## 7. Automatic image `width`/`height` at build (anti-CLS)

**Issue [#13](https://github.com/whoisbrenner/brennercruvinel.com/issues/13) · PR [#14](https://github.com/whoisbrenner/brennercruvinel.com/pull/14)**

Read intrinsic dimensions with Zola's `get_image_metadata()` and emit
`width`/`height` on the image shortcode (colocated and static paths) and the
article banner. Remote/unknown images are skipped via `allow_error`. Added
`height: auto` to the base image rule so the intrinsic ratio is preserved.

**Why.** Images without dimensions cause Cumulative Layout Shift. Setting them by
hand is error-prone and gets skipped — so the dimensions are computed
automatically at build time, never typed manually.

**Files.** `templates/shortcodes/image.html`, `templates/article.html`,
`sass/_media.scss`.

## 8. PWA: manifest, install icons, offline service worker

**Issue [#15](https://github.com/whoisbrenner/brennercruvinel.com/issues/15) · PR [#16](https://github.com/whoisbrenner/brennercruvinel.com/pull/16)**

Add a web app manifest (standalone, theme color, 192/512/180 icons), generated
192×192 and 512×512 install icons, and a conservative network-first service
worker (always fresh online, cached fallback offline, cached home as offline
fallback). Manifest link and SW registration are gated behind
`config.extra.pwa` (enabled by default).

**Why.** The site was not installable and broke entirely offline.

**Files.** `static/site.webmanifest`, `static/sw.js`, `static/register-sw.js`,
`static/icon-192.png`, `static/icon-512.png`, `templates/partials/head.html`,
`config.toml`.

## 9. Demo post archive (2009–2026)

**Issue [#17](https://github.com/whoisbrenner/brennercruvinel.com/issues/17) · PR [#18](https://github.com/whoisbrenner/brennercruvinel.com/pull/18)**

Seed 369 English placeholder posts with random dates/times across the range,
each setting title, date, description, authors, tags and `extra.tldr`. Includes a
reproducible generator and bumps blog pagination to 10.

**Why.** Exercise the new SEO pattern (TL;DR/AEO block, tags, JSON-LD,
breadcrumbs, dates, pagination) end to end and across scale. These are
placeholders to be replaced later by imported content from the old site, so the
generated text is intentionally filler.

**Files.** `content/blog/*.md`, `scripts/generate_demo_posts.py`,
`content/blog/_index.md`.

---

## Notes

- **OG images** use the native Zola pipeline (`resize_image` / static card) by
  decision — the theme stays Node-free. Quartz's Satori-based OG generation was
  not ported.
- The applicable build checks are `zola build` / `zola check` plus browser
  verification (the theme is pure Zola/Tera/SCSS — there is no Rust crate or JS
  lint step).
- `zola check` reports pre-existing broken external links in the original demo
  content (`penandink.work`); these are unrelated to the changes above.
- Process: each change started as a feature request issue and was resolved by a
  single linked pull request, built and verified before merge.
