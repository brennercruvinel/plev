// What the book reads: plev's docs/ (files and CATALOG.json), the monsters'
// blog kept in quartzweb, and the TOML files that get a page of their own.

import fs from "node:fs"
import path from "node:path"

/** every file under `root`, as root-relative paths in lowercase -> real path */
export function scan(root) {
  const files = new Map()
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      if (entry.name.startsWith(".")) continue
      const full = path.join(dir, entry.name)
      if (entry.isDirectory()) walk(full)
      else files.set(path.relative(root, full).split(path.sep).join("/").toLowerCase(), full)
    }
  }
  walk(root)
  return files
}

/**
 * docs/CATALOG.json, the index docbase.py keeps: { byPath, byId }, entries
 * keyed by their docs-relative path in lowercase; `file` keeps the path as
 * git writes it
 */
export function loadCatalog(docs) {
  const byPath = new Map()
  const byId = new Map()
  try {
    const parsed = JSON.parse(fs.readFileSync(path.join(docs, "CATALOG.json"), "utf8"))
    for (const doc of parsed.documents ?? []) {
      if (!doc?.id || !doc?.path) continue
      const entry = { ...doc, file: doc.path, path: doc.path.replace(/^docs\//, "").toLowerCase() }
      byPath.set(entry.path, entry)
      byId.set(entry.id, entry)
    }
  } catch (err) {
    console.warn(`plev book: no usable CATALOG.json (${err.message}); titles come from the files`)
  }
  return { byPath, byId }
}

/** the flat YAML front matter of a blog post: strings, dates and [a, b] lists */
export function frontMatterOf(text) {
  const block = text.match(/^---\n([\s\S]*?)\n---\n?/)
  if (!block) return { data: {}, body: text }
  const data = {}
  for (const line of block[1].split("\n")) {
    const m = line.match(/^([A-Za-z_][\w-]*):\s*(.*)$/)
    if (!m) continue
    let value = m[2].trim()
    if (/^\[.*\]$/.test(value)) {
      value = value
        .slice(1, -1)
        .split(",")
        .map((v) => v.trim().replace(/^["']|["']$/g, ""))
        .filter(Boolean)
    } else value = value.replace(/^["']|["']$/g, "")
    data[m[1]] = value
  }
  return { data, body: text.slice(block[0].length) }
}

/** the blog posts: [{ name, data, body }], index.md first when present */
export function loadBlog(dir) {
  if (!dir || !fs.existsSync(dir)) return []
  return fs
    .readdirSync(dir)
    .filter((n) => n.endsWith(".md"))
    .sort((a, b) => (a === "index.md" ? -1 : b === "index.md" ? 1 : a.localeCompare(b)))
    .map((name) => ({ name, ...frontMatterOf(fs.readFileSync(path.join(dir, name), "utf8")) }))
}

/**
 * A TOML file read as a page: its leading comments as prose, then one
 * section per top-level table ([project], [project.targets] -> "project"),
 * each with its tables as a toml block.
 */
export function tomlSections(text) {
  const lines = text.split("\n")
  const intro = []
  let i = 0
  for (; i < lines.length && !/^\s*\[/.test(lines[i]); i++) intro.push(lines[i])
  const sections = []
  for (; i < lines.length; i++) {
    const head = lines[i].match(/^\s*\[\[?\s*([^\].]+)/)
    const name = head?.[1].trim()
    if (head && name !== sections.at(-1)?.name) sections.push({ name, lines: [] })
    sections.at(-1)?.lines.push(lines[i])
  }
  const prose = intro
    .map((l) => l.replace(/^#\s?/, ""))
    .join("\n")
    .trim()
  return { prose, sections: sections.map((s) => ({ name: s.name, toml: s.lines.join("\n").trim() })) }
}
