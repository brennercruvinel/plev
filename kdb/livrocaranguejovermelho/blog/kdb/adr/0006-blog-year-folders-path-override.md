# 0006. blog in year folders with a path override

- status: accepted
- date: 2026-06-23

## context

about 369 blog posts sat as flat `.md` files in `content/blog/`. i wanted them grouped by year on disk to make the migration from the old site manageable, but without changing any public URL. in Zola, moving a file into a subfolder changes its URL by default, and a URL change breaks accumulated SEO and forces 301 redirects.

## decision

move posts into `content/blog/<year>/`, and in each post's front matter set `path = "blog/<slug>"` to decouple the folder on disk from the public URL. add a transparent section index per year folder (`_index.md` with `transparent = true`, `render = false`) so the posts still belong to the blog section and the listing and pagination keep working, without creating a `/blog/<year>/` page.

## consequences

- posts are grouped by year on disk for the migration.
- every public URL is byte-identical to before, verified by diffing the built URL set against a baseline.
- the year never leaks into the URL.
- every post now carries an explicit `path`, which is the price of the decoupling.
- page bundles that use i18n symlinks were left where they were; a fixed `path` would break their multilingual routing.
