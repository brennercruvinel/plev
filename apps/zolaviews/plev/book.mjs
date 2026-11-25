#!/usr/bin/env node
// plev book for Zola: writes plev's docs/ (and the monsters' blog) as Zola
// content for the Ametrine theme, reading them the way apps/quartzweb's
// plev-book plugin does.
//
//   docs/README.md          -> content/_index.md, the home page
//   docs/<dir>/<dir>.md     -> content/<dir>/_index.md: the folder's note (or
//   docs/<dir>/README.md       a README) is a collection (an index: cards of
//                              what it holds, its written index folded below)
//                              or a document (a task or an ADR: its text, then
//                              cards of its phases or its learnings)
//   a folder of pages without a note -> a generated collection
//   docs/<path>.md          -> content/<path>.md, an article (paths in lowercase)
//   any other file          -> static/docs/<path>, served at /docs/<path>
//   ATLAS.toml, TERMS.toml  -> content/atlas.md, content/terms.md, one
//                              section per table
//   quartzweb content/blog  -> content/blog
//
// From docs/CATALOG.json each page gets its title in sentence case, the
// first paragraph as description, date and updated, topics as tags, its
// status as a category (with the status' accent color when it asks for
// attention; a superseded record is marked archived), relations as a
// "Related" list, and a weight that orders it among its siblings (newest
// first by the date of the record; a document's phases in file order).
// The docs are not changed.
//
//   node plev/book.mjs           write content/ and static/docs/ once
//   node plev/book.mjs --watch   write, then again on every change
//   node plev/book.mjs --serve   watch and run `zola serve` (other args go to zola)
//
// PLEV_DOCS (../../docs), PLEV_BOOK (../quartzweb/plugins/plev-book/src)
// and PLEV_BLOG (../quartzweb/content/blog) point elsewhere when needed.

import fs from "node:fs"
import path from "node:path"
import { spawn } from "node:child_process"
import { fileURLToPath, pathToFileURL } from "node:url"
import { escapeTemplates, leadParagraph, textTools } from "./markdown.mjs"
import { frontMatterOf, loadBlog, loadCatalog, scan, tomlSections } from "./sources.mjs"

const SITE = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..")
const DOCS = path.resolve(SITE, process.env.PLEV_DOCS ?? "../../docs")
const BOOK = path.resolve(SITE, process.env.PLEV_BOOK ?? "../quartzweb/plugins/plev-book/src")
const BLOG = path.resolve(SITE, process.env.PLEV_BLOG ?? "../quartzweb/content/blog")
const REPO = "https://github.com/brennercruvinel/plev/blob/main"

for (const [what, dir] of [["docs", DOCS], ["plev-book", BOOK]]) {
  if (!fs.existsSync(dir)) {
    console.error(`plev book: ${what} not found at ${dir}`)
    process.exit(1)
  }
}
const load = (name) => import(pathToFileURL(path.join(BOOK, name)).href)
const casing = await load("casing.js")
const { SECTIONS } = await load("sections.js")
const { outsideLink, targetPath } = await load("links.js")
const { body } = textTools(casing)
const { pageTitle, readable } = casing

// a status is a category; the ones that ask for attention tint the page
const STATUS = {
  accepted: { label: "Accepted" },
  proposed: { label: "Proposed", accent: "yellow" },
  superseded: { label: "Superseded", accent: "red" },
  done: { label: "Done" },
  "in-progress": { label: "In progress", accent: "blue" },
  pending: { label: "Pending", accent: "orange" },
  active: { label: "Active" },
  reference: { label: "Reference" },
}
// what a folder holds, by the type of most of its pages
const NOUNS = {
  adr: "decisions",
  task: "tasks",
  reference: "references",
  research: "notes",
  experiment: "experiments",
  learning: "learnings",
  manual: "manual pages",
}
// the non-markdown files that get a page of their own
const EMBEDS = { "atlas.toml": ["atlas.md", "Atlas"], "terms.toml": ["terms.md", "Terms"] }
// the home lists the book in the order a reader meets it
const HOME_ORDER = ["changelog.md", "blog", "tasks", "adr", "research", "atlas.md", "terms.md"]

// a folder's note: its README, or the page named like the folder (adr/adr.md,
// tasks/task-01-x/task-01-x.md), the shape docs/ uses since 2026-11
const isReadme = (p) =>
  /(^|\/)readme\.md$/.test(p) || (p.includes("/") && path.posix.basename(p, ".md") === path.posix.basename(parentOf(p)))
const noteOf = (dir, catalog) => catalog.byPath.get(`${dir}/readme.md`) ?? catalog.byPath.get(`${dir}/${path.posix.basename(dir)}.md`)
// the Related list docbase.py writes into every page; the book draws its own from the relations
const withoutRelated = (text) => text.replace(/\n*<!-- related:[\s\S]*?<!-- \/related -->\n?/, "\n")
// a TEMPLATE.md is the form a new record starts from; links to it open the forge
const isTemplate = (p) => /(^|\/)template\.md$/.test(p)
const outPath = (p) => (isReadme(p) ? `${parentOf(p) === "." ? "" : `${parentOf(p)}/`}_index.md` : p)
const isDay = (v) => typeof v === "string" && /^\d{4}-\d{2}-\d{2}$/.test(v) && !Number.isNaN(Date.parse(v))
const parentOf = (dir) => path.posix.dirname(dir)

// front matter --------------------------------------------------------------

const str = (v) => JSON.stringify(String(v))
const tomlValue = (v) => (Array.isArray(v) ? `[${v.map(str).join(", ")}]` : typeof v === "string" ? str(v) : String(v))

function frontMatter(fields, extra = {}, taxonomies = {}) {
  const lines = ["+++"]
  const put = (obj) => {
    for (const [k, v] of Object.entries(obj)) {
      if (v === undefined || v === null || v === "" || (Array.isArray(v) && v.length === 0)) continue
      lines.push(`${k} = ${tomlValue(v)}`)
    }
  }
  put(fields)
  if (Object.values(taxonomies).some((v) => v?.length)) {
    lines.push("", "[taxonomies]")
    put(taxonomies)
  }
  lines.push("", "[extra]")
  put(extra)
  lines.push("+++", "")
  return lines.join("\n")
}

// links ---------------------------------------------------------------------

function makeLinker(here, files) {
  return (url) => {
    const away = outsideLink(here, url, REPO)
    if (away !== url) return away
    const target = targetPath(here, url)?.toLowerCase()
    if (!target) return url
    const hash = url.includes("#") ? url.slice(url.indexOf("#")) : ""
    if (EMBEDS[target]) return `@/${EMBEDS[target][0]}${hash}`
    if (isTemplate(target)) return `${REPO}/docs/${target.replace(/template\.md$/, "TEMPLATE.md")}${hash}`
    if (!files.has(target)) return url
    if (target.endsWith(".md")) return `@/${contentOf.get(target) ?? outPath(target)}${hash}`
    return `/docs/${target}${hash}`
  }
}

// the tree ------------------------------------------------------------------

/** the docs as nodes: pages, sections (from a README or generated) and the home */
function nodesOf(files, catalog) {
  const nodes = []
  // a folder is a section when pages sit under it; a task or experiment
  // folder with only its README is one page (a Zola page bundle)
  const sectionDirs = new Set()
  const readmeDirs = new Set()
  for (const p of files.keys()) {
    if (!p.endsWith(".md") || isTemplate(p)) continue
    if (isReadme(p)) readmeDirs.add(parentOf(p))
    else for (let d = parentOf(p); d !== "."; d = parentOf(d)) sectionDirs.add(d)
  }
  for (const dir of readmeDirs) {
    const type = noteOf(dir, catalog)?.type
    if (dir !== "." && (type === "index" || sectionDirs.has(dir))) {
      for (let d = dir; d !== "."; d = parentOf(d)) sectionDirs.add(d)
    }
  }
  for (const [p, real] of files) {
    if (!p.endsWith(".md") || isTemplate(p)) continue
    const text = withoutRelated(fs.readFileSync(real, "utf8"))
    const entry = catalog.byPath.get(p) ?? ownEntry(p, real, text)
    const dir = parentOf(p)
    const fallback = path.posix.basename(p, ".md").replace(/[-_]/g, " ")
    const original = entry?.title ?? text.match(/^title:\s*"?(.*?)"?\s*$/m)?.[1] ?? fallback
    const top = isReadme(p) && SECTIONS[dir]
    const node = { p, real, entry, text, original, title: top || pageTitle(original, { id: entry?.id, type: entry?.type }) }
    node.date = isDay(entry?.date) ? entry.date : isDay(entry?.recorded) ? entry.recorded : undefined
    node.updated = isDay(entry?.revised) && entry.revised !== node.date ? entry.revised : undefined
    if (p === "readme.md") Object.assign(node, { kind: "home", dir: ".", title: "plev" })
    else if (isReadme(p) && sectionDirs.has(dir)) {
      const kind = entry?.type === "index" ? "collection" : "document"
      Object.assign(node, { kind, dir, parent: parentOf(dir) })
    } else if (isReadme(p)) Object.assign(node, { kind: "page", bundle: true, parent: parentOf(dir) })
    else Object.assign(node, { kind: "page", parent: dir })
    nodes.push(node)
  }
  for (const dir of sectionDirs) {
    if (readmeDirs.has(dir)) continue
    const title = SECTIONS[dir] ?? readable(path.posix.basename(dir).replace(/[-_]/g, " "))
    nodes.push({ kind: "collection", generated: true, dir, parent: parentOf(dir), title })
  }
  return nodes
}

/**
 * A page CATALOG.json does not list yet (docbase.py runs later): its own
 * front matter, with the status' first word ("done.", "in-progress, ...")
 */
function ownEntry(p, real, text) {
  const { data } = frontMatterOf(text)
  if (!data.title && !data.date) return undefined
  const status = String(data.status ?? "").toLowerCase().match(/^[a-z-]+/)?.[0]
  const file = `docs/${path.relative(DOCS, real).split(path.sep).join("/")}`
  return { ...data, status, path: p, file, topics: Array.isArray(data.topics) ? data.topics : [], relations: Array.isArray(data.relations) ? data.relations : [] }
}

function contentPath(n) {
  if (n.bundle) return `${parentOf(n.p)}/index.md`
  return n.kind === "page" ? outPath(n.p) : n.kind === "home" ? "_index.md" : `${n.dir}/_index.md`
}
// docs path -> content path, for links and relations
let contentOf = new Map()
const recency = (n) => n.updated ?? n.date ?? ""

/** children of each section, ordered: newest first, a document's phases in file order */
function order(nodes) {
  const byDir = new Map()
  for (const n of nodes) {
    if (n.kind === "home") continue
    if (!byDir.has(n.parent)) byDir.set(n.parent, [])
    byDir.get(n.parent).push(n)
  }
  const sections = new Map(nodes.filter((n) => n.dir !== undefined).map((n) => [n.dir, n]))
  const homeRank = (n) => {
    const i = HOME_ORDER.indexOf(n.kind === "page" ? path.posix.basename(n.p) : n.dir)
    return i < 0 ? HOME_ORDER.length : i
  }
  for (const [dir, kids] of byDir) {
    const owner = sections.get(dir)
    if (owner?.kind === "document") kids.sort((a, b) => contentPath(a).localeCompare(contentPath(b)))
    else if (dir === ".") kids.sort((a, b) => homeRank(a) - homeRank(b) || a.title.localeCompare(b.title))
    else kids.sort((a, b) => (b.date ?? "").localeCompare(a.date ?? "") || a.title.localeCompare(b.title))
    kids.forEach((k, i) => (k.weight = i + 1))
    if (owner) owner.children = kids
  }
}

function related(entry, here, catalog) {
  if (!Array.isArray(entry?.relations) || entry.type === "index") return ""
  const seen = new Set()
  const items = []
  for (const id of entry.relations) {
    const t = catalog.byId.get(id)
    if (!t || seen.has(id) || t.path === here) continue
    seen.add(id)
    const section = isReadme(t.path) && SECTIONS[parentOf(t.path)]
    const title = section || pageTitle(t.title ?? t.id, { id: t.id, type: t.type })
    items.push(`- [${title}](@/${contentOf.get(t.path) ?? outPath(t.path)})`)
  }
  return items.length ? `\n## Related\n\n${items.join("\n")}\n` : ""
}

/** "11 decisions.": what a section holds, by the type of most of its pages */
function describe(n) {
  const count = n.children?.length ?? 0
  const tally = {}
  for (const k of n.children ?? []) if (k.entry?.type) tally[k.entry.type] = (tally[k.entry.type] ?? 0) + 1
  const top = Object.entries(tally).sort((a, b) => b[1] - a[1])[0]?.[0]
  const noun = NOUNS[top] ?? (count === 1 ? "page" : "pages")
  const latest = [...(n.children ?? [])].filter((k) => k.date).sort((a, b) => b.date.localeCompare(a.date))[0]
  return latest ? `${count} ${noun}; the latest, ${latest.date}: *${latest.title}*.` : `${count} ${noun}.`
}

/** the Zola file of one docs node */
function emit(n, files, catalog, recent) {
  const children = (n.children ?? []).map(contentPath)
  if (n.generated) {
    const fields = { title: n.title, description: describe(n), sort_by: "weight", weight: n.weight }
    return frontMatter(fields, { kind: "collection", count: children.length, children })
  }
  const entry = n.entry
  const status = STATUS[entry?.status]
  const meta = {
    id: entry?.id,
    type: entry?.type,
    status: entry?.status,
    date_basis: entry?.date_basis,
    review: entry?.review,
    evidence: entry?.evidence,
    source: entry?.file ?? `docs/${path.relative(DOCS, n.real).split(path.sep).join("/")}`,
    accent_color: status?.accent,
  }
  const text = body(n.text, { original: n.original, link: makeLinker(n.p, files) })
  const { lead } = leadParagraph(text)
  // the page of a collection (and the home) opens with its whole opening
  // paragraph, and the rest of its README is its written index
  const opening = leadParagraph(text, 4000, true)
  const intro = n.kind === "home" || n.kind === "collection" ? opening.lead : undefined
  const rest = opening.rest
  const more = related(entry, n.p, catalog)
  if (n.kind === "page") {
    const fields = { title: n.title, description: lead, date: n.date, updated: n.updated, weight: n.weight, template: "article.html" }
    if (entry?.status === "superseded") meta.archive = "A later record superseded this one; the related records below say which."
    return frontMatter(fields, meta, { tags: entry?.topics, categories: status ? [status.label] : [] }) + text + more
  }
  const extra = { kind: n.kind, intro, date: n.date, updated: n.updated, status_label: status?.label, topics: entry?.topics, count: children.length, children, ...meta }
  if (n.kind === "home") return frontMatter({ title: n.title, description: lead }, { ...extra, recent }) + rest
  const fields = { title: n.title, description: lead || describe(n), sort_by: "weight", weight: n.weight }
  // a collection shows its written index folded under the cards; a document is read as is
  return frontMatter(fields, extra) + (n.kind === "collection" ? rest : text + more)
}

function blogFiles(out) {
  const posts = loadBlog(BLOG)
  if (posts.length === 0) return
  const index = posts.find((p) => p.name === "index.md")
  const items = posts.filter((p) => p !== index && isDay(String(p.data.date ?? "")))
  items.sort((a, b) => String(b.data.date).localeCompare(String(a.data.date)))
  const asIs = (text, title) => body(text, { original: title, link: (u) => u, keepCase: true })
  items.forEach((post, i) => {
    const { title, description, date, author, tags } = post.data
    const fields = { title, description, date, weight: i + 1, template: "article.html", authors: author ? [author] : undefined }
    const extra = { source: `apps/quartzweb/content/blog/${post.name}` }
    out.set(path.join("content", "blog", post.name), escapeTemplates(frontMatter(fields, extra, { tags }) + asIs(post.body, title)))
  })
  // the blog's own index text opens its page whole; the posts follow as cards
  const intro = index ? asIs(index.body, index.data.title).trim() : ""
  const fields = { title: "Blog", description: index?.data.description ?? leadParagraph(intro).lead, sort_by: "weight", weight: 2 }
  const extra = { kind: "collection", intro: intro || undefined, count: items.length, children: items.map((p) => `blog/${p.name}`) }
  out.set(path.join("content", "blog", "_index.md"), escapeTemplates(frontMatter(fields, extra)))
}

function embedFiles(out, files) {
  for (const [file, [page, title]] of Object.entries(EMBEDS)) {
    const real = files.get(file)
    if (!real) continue
    const { prose, sections } = tomlSections(fs.readFileSync(real, "utf8"))
    const name = path.basename(real)
    const said = body(prose, { link: (u) => u })
    const parts = sections.map((s) => `## ${readable(s.name)}\n\n\`\`\`toml\n${s.toml}\n\`\`\`\n`).join("\n")
    const text = `${said}\nRead from [docs/${name}](${REPO}/docs/${name}).\n\n${parts}`
    const fields = { title, description: leadParagraph(said).lead, template: "article.html" }
    out.set(path.join("content", page), escapeTemplates(frontMatter(fields, { source: `docs/${name}` }) + text))
  }
}

function build() {
  const files = scan(DOCS)
  const catalog = loadCatalog(DOCS)
  const nodes = nodesOf(files, catalog)
  contentOf = new Map(nodes.filter((n) => n.p).map((n) => [n.p, contentPath(n)]))
  order(nodes)
  const recent = nodes
    .filter((n) => (n.kind === "page" || n.kind === "document") && recency(n))
    .sort((a, b) => recency(b).localeCompare(recency(a)))
    .slice(0, 6)
    .map(contentPath)
  const out = new Map()
  for (const n of nodes) out.set(path.join("content", contentPath(n)), escapeTemplates(emit(n, files, catalog, recent)))
  for (const [p, real] of files) {
    if (!p.endsWith(".md") && !EMBEDS[p] && p !== "catalog.json") out.set(path.join("static", "docs", p), { copy: real })
  }
  blogFiles(out)
  embedFiles(out, files)
  return out
}

// writing -------------------------------------------------------------------

/** write what changed, remove what is gone; Zola only rebuilds what it sees change */
function write(out) {
  let written = 0
  let removed = 0
  for (const [rel, value] of out) {
    const dest = path.join(SITE, rel)
    const data = typeof value === "string" ? Buffer.from(value) : fs.readFileSync(value.copy)
    if (fs.existsSync(dest) && fs.readFileSync(dest).equals(data)) continue
    fs.mkdirSync(path.dirname(dest), { recursive: true })
    fs.writeFileSync(dest, data)
    written++
  }
  for (const root of [path.join(SITE, "content"), path.join(SITE, "static", "docs")]) {
    if (!fs.existsSync(root)) continue
    const sweep = (dir) => {
      for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const full = path.join(dir, entry.name)
        if (entry.isDirectory()) {
          sweep(full)
          if (fs.readdirSync(full).length === 0) fs.rmdirSync(full)
        } else if (!out.has(path.relative(SITE, full))) {
          fs.unlinkSync(full)
          removed++
        }
      }
    }
    sweep(root)
  }
  return { written, removed, total: out.size }
}

function run() {
  const t = performance.now()
  const result = write(build())
  const { written, removed, total } = result
  console.log(`plev book: ${total} files (${written} written, ${removed} removed, ${Math.round(performance.now() - t)} ms)`)
  return result
}

// zola serve rebuilds the whole site once per changed file and answers 404
// meanwhile; past this many changes a fresh start (one build) is faster
const RESTART_AFTER = 20
const args = process.argv.slice(2)
const serving = args.includes("--serve")
let zola = null
let stopping = false

function startZola() {
  const rest = args.filter((a) => a !== "--serve" && a !== "--watch")
  zola = spawn("zola", ["serve", ...rest], { cwd: SITE, stdio: "inherit" })
  zola.on("exit", (code) => {
    if (stopping) process.exit(code ?? 0)
  })
}

function restartZola() {
  if (!zola) return startZola()
  const old = zola
  old.removeAllListeners("exit")
  old.once("exit", startZola)
  old.kill("SIGTERM")
}

run()
if (args.includes("--watch") || serving) {
  let timer
  const again = () => {
    clearTimeout(timer)
    timer = setTimeout(() => {
      try {
        const { written, removed } = run()
        if (serving && written + removed > RESTART_AFTER) restartZola()
      } catch (err) {
        console.error(`plev book: ${err.message}`)
      }
    }, 150)
  }
  fs.watch(DOCS, { recursive: true }, again)
  if (fs.existsSync(BLOG)) fs.watch(BLOG, again)
  console.log("plev book: watching docs/ and the blog")
}
if (serving) {
  startZola()
  for (const sig of ["SIGINT", "SIGTERM"]) {
    process.on(sig, () => {
      stopping = true
      zola?.kill(sig)
    })
  }
}
