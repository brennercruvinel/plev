# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

A **collection of 96 independent frontend demo projects** — not a monorepo. Each subdirectory is self-contained with its own dependencies and build setup.

## Serving the Gallery

```bash
npm start          # ou npm run dev — Node.js server (serve.js)
```

Auto-detects free port starting from 8080. Displays URL in terminal. The `serve.js` also provides a `POST /api/import` endpoint for the smart import system.

The gallery has embedded project data (no external fetch), localStorage-based editing (key `demosfront_v3`), search/filter by category/tech, 3 view modes (grid/list/compact), 3 thumbnail sizes (S/M/L), and smart import via drawer. The canonical data source is `demos.json`. Real screenshots in `thumbs/` generated via Playwright.

The root `src/` and `dist/` directories are **leftover artifacts from an unrelated CodePen port** — they are not part of the gallery.

## Smart Import System

The "+" button opens a right-side drawer. Paste a GitHub or CodePen URL:

- **GitHub**: `git clone --depth 1` → detect pkg → fix vite base → `npm install && build` → fix absolute paths → replace author → generate thumb → save to `demos.json`
- **CodePen**: Playwright stealth headless (bypasses Cloudflare) → extract HTML/CSS/JS from CodeMirror → create dist/src → detect tech → generate thumb from local HTML via `file://` → save

**Duplicate detection**: if `dir` already exists in `demos.json` and files are on disk, skips extraction and returns existing data instantly.

**Author replacement**: all imported projects get author set to `brenner@hoffresearch.com` in package.json.

Backend streams progress via SSE (`text/event-stream`). Frontend renders steps in real-time with spinner.

Key files: `serve.js` (server + import API), `.codepen-extract.py` (generated at runtime for CodePen extraction).

## Project Categories

### CodePen Ports (~54 projects)
Pure HTML/CSS/JS — no package.json, no build step. Structure: `src/` and `dist/` containing `index.html`, `script.js`, `style.css`. Some include bundled libs in `libs/` (GSAP, Anime.js, D3.js). Heavy use of Canvas API, raw WebGL, CSS animations. CodePen imports use `type="module"` in script tag when ES imports are detected.

### Landing Pages (~12 projects)
React+Vite projects with landing page layout (hero, features, CTA, footer). Same build tooling as React projects but categorized separately: `2586-labs`, `agentik`, `aueik`, `barber`, `design-courses`, `devfolio`, `genos`, `nduz-ai`, `pastel-portfolio`, `rhinos-gym`, `tars`, `triomix`.

### React + Vite (~14 projects)
```bash
cd <project>
npm install
npm run build    # Production build → dist/
```

- TypeScript + React + Vite
- Path alias: `@/` → `src/`
- Common deps: Three.js, R3F, GSAP, Framer Motion, Tailwind CSS, Lucide React
- All vite.config files have `base: "./"` for relative paths (critical for subdirectory serving)
- Some have `.env.local` for `GEMINI_API_KEY`
- `slice-viewer` is the only one with ESLint and CI/CD

### Codrops (3 projects)
`ContextAwareLogoAnimationScroll`, `ScrollTextMotion`, `Staggered3DGridAnimations` — plain HTML with bundled GSAP/Lenis/ScrollTrigger in `js/libs/`. Serve statically. `Staggered3DGridAnimations` requires GSAP Club license for SplitText.

### Astro (2 projects)
`gsap-threejs-codrops`, `aiengs` — standard Astro: `npm install && npm run build` (output in `dist/`).

### Webpack (1 project)
`threejs-depth-points-image` — build output goes to `build/` (not `dist/`).

### Custom WebGL (2 projects)
`Blurry` (custom shader pipeline with curl noise) and `sketch492` (Vite+TS, Three.js + Matter.js, no React). Both serve statically.

### Haxe (`works/` — 8 projects)
`bubbles`, `chill`, `clock`, `drops`, `jelly`, `life`, `marimo`, `water` — require Haxe compiler v4.3.1+ and libs from `github.com/saharan/haxelibs/`. Build: `haxe build.hxml`. Output in `bin/`. Not Node.js-based.

## Path Conventions in demos.json

Each entry has `dir` (directory name) and `path` (build output subfolder):
- `"path": "dist"` → link to `{dir}/dist/index.html` (most projects)
- `"path": "build"` → `{dir}/build/index.html` (webpack)
- `"path": "bin"` → `{dir}/bin/index.html` (haxe)
- `"path": "."` → `{dir}/index.html` (codrops, Blurry)

## Critical Build Rules

- **Always set `base: "./"` in vite.config** — without it, Vite generates absolute paths (`/assets/...`) that break when served from subdirectories
- After building any Vite/Astro/Webpack project, verify `dist/index.html` has **no absolute paths** in `src` or `href` attributes (should be `./assets/...` not `/assets/...`)
- Each project is independent — changes to one must never affect another
- Most subdirectories have their own `.git`; the root directory is not a git repo
