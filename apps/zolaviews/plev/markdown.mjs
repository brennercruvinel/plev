// The text of a page: links rewritten for Zola, headings and running text in
// sentence case (plev-book's rules), template syntax escaped, and the first
// paragraph taken as the page's description.

const INLINE = /(\]\()(<[^>]+>|[^)\s]+)((?:\s+"[^"]*")?\))/g
const DEFINITION = /^(\s*\[[^\]]+\]:\s*)(\S+)(.*)$/
// list marker, task box and blockquote marks in front of a block's text
const MARKER = /^(\s*(?:(?:[-*+]|\d+[.)])\s+(?:\[[ xX]\]\s+)?|>\s*)*)(.*)$/
const BLOCK_OPEN = /^\s*(?:[-*+]|\d+[.)])\s+|^\s*>/

// Zola 0.23 reads {{ }} and {% %} as template code everywhere, code blocks
// included; each opener is written as a template string that prints it. The
// TOML front matter of a Zola file is not template code and stays as it is.
export function escapeTemplates(text) {
  const head = text.startsWith("+++\n") ? text.indexOf("\n+++\n", 3) + 5 : 0
  return text.slice(0, head) + text.slice(head).replace(/\{\{|\{%/g, (open) => `{{ "${open}" }}`)
}

export function stripFrontMatter(text) {
  return text.replace(/^---\n[\s\S]*?\n---\n?/, "")
}

/** the text tools, given plev-book's casing functions */
export function textTools({ capitalizeFirst, readable, sameText, sentenceStarts }) {
  /** the links of one line, outside its inline code spans; prose also gets sentence starts */
  function relink(line, link, prose) {
    return line
      .split(/(`[^`]*`)/)
      .map((part, i) => {
        if (i % 2) return part
        const linked = part.replace(INLINE, (_, open, url, close) => open + link(url.replace(/^<|>$/g, "")) + close)
        return prose ? sentenceStarts(linked) : linked
      })
      .join("")
  }

  // a block (paragraph, list item, table cell) that opens with a word starts
  // with a capital; one that opens with a link, code or emphasis stays
  function capitalizeBlock(line) {
    const [, marker, rest] = line.match(MARKER)
    return /^\p{L}/u.test(rest) ? marker + capitalizeFirst(rest) : line
  }

  function caseLine(line, starts) {
    if (/^\s*</.test(line)) return line
    if (/^\s*\|/.test(line)) {
      if (/^[\s|:-]+$/.test(line)) return line
      return line
        .split("|")
        .map((cell) => {
          const lead = cell.match(/^\s*/)[0]
          return lead + capitalizeBlock(cell.slice(lead.length))
        })
        .join("|")
    }
    return starts || BLOCK_OPEN.test(line) ? capitalizeBlock(line) : line
  }

  /**
   * The body of a docs page: links through `link`, headings and prose in
   * sentence case unless `keepCase`, and a first "# title" equal to
   * `original` dropped (the page shows its title).
   */
  function body(text, { original, link, keepCase = false }) {
    const out = []
    let fence = null
    let dropped = false
    for (const line of stripFrontMatter(text).split("\n")) {
      const f = line.match(/^\s*(```+|~~~+)/)
      if (f || fence) {
        if (f && !fence) fence = f[1][0]
        else if (f && f[1][0] === fence) fence = null
        out.push(line)
        continue
      }
      const h = line.match(/^(#{1,6})\s+(.*)$/)
      if (h) {
        const first = h[1] === "#" && out.every((l) => l.trim() === "")
        if (!dropped && first && original && sameText(h[2], original)) {
          dropped = true
          continue
        }
        out.push(`${h[1]} ${relink(keepCase ? h[2] : readable(h[2]), link, false)}`)
        continue
      }
      const d = line.match(DEFINITION)
      if (d) {
        out.push(d[1] + link(d[2]) + d[3])
        continue
      }
      if (keepCase) {
        out.push(relink(line, link, false))
        continue
      }
      const prev = out.length ? out[out.length - 1].trim() : ""
      // a block starts after a blank line or a heading; a sentence also starts after a line that ended one
      const starts = prev === "" || /^#{1,6}\s/.test(prev) || (/[.!?]$/.test(prev) && !/(?:e\.g|i\.e|vs|etc|cf)\.$/i.test(prev))
      out.push(relink(caseLine(line, starts), link, !/^\s*</.test(line)))
    }
    return out.join("\n").replace(/\s+$/, "") + "\n"
  }

  return { body }
}

/**
 * The first paragraph of a body, as plain text for a card (links become
 * their text, at most `max` characters), and the body without it. With
 * `opening`, only a paragraph before the first heading counts.
 */
export function leadParagraph(markdown, max = 220, opening = false) {
  const lines = markdown.split("\n")
  let start = -1
  let fence = false
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]
    if (/^\s*(```|~~~)/.test(line)) fence = !fence
    if (fence || line.trim() === "") continue
    if (/^\s*(#|\||<|>|[-*+]\s|\d+[.)]\s|!\[|\[\^|---|\{\{)/.test(line)) {
      if (opening && /^\s*#/.test(line)) break
      continue
    }
    start = i
    break
  }
  if (start < 0) return { lead: "", rest: markdown }
  let end = start
  while (end < lines.length && lines[end].trim() !== "") end++
  const raw = lines.slice(start, end).join(" ")
  let lead = raw
    .replace(/!\[[^\]]*\]\([^)]*\)/g, "")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\[\^[^\]]*\]/g, "")
    .replace(/[*_]{1,3}([^*_]+)[*_]{1,3}/g, "$1")
    .replace(/\{\{ "(\{[{%])" \}\}/g, "$1")
    .replace(/\s+/g, " ")
    .trim()
  if (lead.length > max) lead = lead.slice(0, lead.lastIndexOf(" ", max)).replace(/[,;:]$/, "") + "…"
  const rest = [...lines.slice(0, start), ...lines.slice(end)].join("\n").replace(/^\s+/, "")
  return { lead, rest }
}
