#!/usr/bin/env python3
"""
Generate a demonstrative blog archive for the Brenner theme.

Creates N english posts with random dates/times spread across a date range,
each exercising the new SEO pattern: title, date, description, authors, tags
and an extra.tldr (the AEO summary block). These are placeholder posts meant
to be replaced later by imported content from the old site.

Deterministic: a fixed seed makes the output reproducible.

Usage:
    python3 scripts/generate_demo_posts.py [count]
"""

import os
import random
import sys
import unicodedata
from datetime import datetime, timedelta

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_DIR = os.path.join(ROOT, "content", "blog")

SEED = 20260619
COUNT = int(sys.argv[1]) if len(sys.argv) > 1 else 369
START = datetime(2009, 1, 1, 0, 0, 0)
END = datetime(2026, 6, 19, 23, 59, 59)
AUTHOR = "Brenner Cruvinel"

# topic -> (tags, subjects, angle pool)
TOPICS = {
    "programming": (
        ["programming", "engineering"],
        ["clean functions", "error handling", "naming things", "code review",
         "refactoring", "pure functions", "data structures", "recursion",
         "concurrency", "immutability", "testing", "debugging"],
    ),
    "web": (
        ["web", "frontend"],
        ["semantic html", "css grid", "progressive enhancement", "accessibility",
         "web performance", "core web vitals", "responsive design", "forms",
         "service workers", "the cascade", "design tokens", "view transitions"],
    ),
    "backend": (
        ["backend", "systems"],
        ["caching", "rate limiting", "database indexes", "queues", "idempotency",
         "pagination", "observability", "feature flags", "schema migrations",
         "api design", "background jobs", "connection pools"],
    ),
    "devops": (
        ["devops", "infra"],
        ["reproducible builds", "ci pipelines", "blue-green deploys", "rollbacks",
         "infrastructure as code", "secrets management", "log aggregation",
         "health checks", "zero-downtime deploys", "container images"],
    ),
    "design": (
        ["design", "ux"],
        ["typography", "whitespace", "visual hierarchy", "color contrast",
         "micro-interactions", "empty states", "loading states", "iconography",
         "grid systems", "dark mode", "motion design"],
    ),
    "writing": (
        ["writing", "essays"],
        ["writing clearly", "editing ruthlessly", "the first sentence",
         "killing adverbs", "structure before style", "writing for skimmers",
         "the rewrite", "notes to self", "thinking on paper"],
    ),
    "productivity": (
        ["productivity", "workflow"],
        ["deep work", "saying no", "small commits", "single tasking",
         "weekly reviews", "inbox zero", "plain text everything",
         "keyboard over mouse", "automating the boring parts"],
    ),
    "seo": (
        ["seo", "geo"],
        ["entity seo", "structured data", "answer engines", "earned media",
         "zero-click search", "the knowledge graph", "schema graphs",
         "speakable content", "proprietary data", "citations over rankings"],
    ),
    "linux": (
        ["linux", "cli"],
        ["the shell", "pipes and filters", "tmux", "ssh keys", "cron jobs",
         "dotfiles", "ripgrep", "process substitution", "systemd units"],
    ),
}

TITLE_PATTERNS = [
    "on {s}",
    "notes on {s}",
    "a short note about {s}",
    "thinking about {s}",
    "{s}, revisited",
    "the case for {s}",
    "what i learned about {s}",
    "{s} in practice",
    "a small guide to {s}",
    "rethinking {s}",
]

INTROS = [
    "I keep coming back to {s}, so here are the notes I wish I had earlier.",
    "Every few months {s} bites me again. This is what finally stuck.",
    "Short post: {s} is one of those things that looks simple until it isn't.",
    "I spent a while getting {s} wrong. Here is the version that works for me.",
    "A quick write-up on {s}, mostly so future me stops relearning it.",
]

POINTS = [
    "Start with the smallest thing that works, then make it boring on purpose.",
    "Optimise for the reader who shows up six months from now with no context.",
    "Most of the value is in deleting, not adding.",
    "Measure before you change anything; intuition is a poor profiler.",
    "Make the default path the correct path so nobody has to remember the rule.",
    "Name it after what it does, not after how it does it.",
    "If it is hard to test, that is the design telling you something.",
    "Prefer the obvious solution; clever costs interest forever.",
    "Write it down once, link to it everywhere.",
    "The fix that survives is the one the next person can understand.",
]

TLDRS = [
    "Keep {s} small, explicit and boring; the clever version costs you later.",
    "The practical rule for {s}: measure first, delete second, document third.",
    "{s} gets easier when the default path is the correct one.",
    "Most {s} problems are really naming and structure problems in disguise.",
    "Treat {s} as something the next person has to read, not just run.",
]


def slugify(text):
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode()
    text = text.lower()
    out = []
    for ch in text:
        if ch.isalnum():
            out.append(ch)
        elif ch in " -_":
            out.append("-")
    slug = "".join(out)
    while "--" in slug:
        slug = slug.replace("--", "-")
    return slug.strip("-")


def random_datetime(rng):
    delta = int((END - START).total_seconds())
    return START + timedelta(seconds=rng.randint(0, delta))


def main():
    rng = random.Random(SEED)
    os.makedirs(OUT_DIR, exist_ok=True)
    used = set()
    written = 0

    for i in range(COUNT):
        topic = rng.choice(list(TOPICS.keys()))
        tags, subjects = TOPICS[topic]
        subject = rng.choice(subjects)
        title = rng.choice(TITLE_PATTERNS).format(s=subject)

        slug = slugify(title)
        base_slug = slug
        n = 2
        while slug in used:
            slug = f"{base_slug}-{n}"
            n += 1
        used.add(slug)

        dt = random_datetime(rng)
        date_str = dt.strftime("%Y-%m-%dT%H:%M:%SZ")

        post_tags = list(dict.fromkeys(tags + [subject]))
        tldr = rng.choice(TLDRS).format(s=subject)
        intro = rng.choice(INTROS).format(s=subject)
        chosen_points = rng.sample(POINTS, k=3)

        body = []
        body.append(intro)
        body.append("")
        body.append("## why it matters")
        body.append("")
        body.append(
            f"{subject.capitalize()} shows up across {tags[0]} work more often than it "
            "deserves credit for. The details are small, but they compound."
        )
        body.append("")
        body.append("## what works for me")
        body.append("")
        for p in chosen_points:
            body.append(f"- {p}")
        body.append("")
        body.append(
            "None of this is a law. It is just the shape that kept working after "
            "enough mistakes."
        )
        body.append("")

        # escape double quotes for TOML basic strings
        def toml_str(value):
            return value.replace("\\", "\\\\").replace('"', '\\"')

        front = []
        front.append("+++")
        front.append(f'title = "{toml_str(title)}"')
        front.append(f"date = {date_str}")
        front.append(f'description = "{toml_str(tldr)}"')
        front.append(f'authors = ["{toml_str(AUTHOR)}"]')
        front.append("")
        front.append("[taxonomies]")
        tags_toml = ", ".join(f'"{toml_str(t)}"' for t in post_tags)
        front.append(f"tags = [{tags_toml}]")
        front.append("")
        front.append("[extra]")
        front.append(f'tldr = "{toml_str(tldr)}"')
        front.append("+++")
        front.append("")

        content = "\n".join(front) + "\n".join(body) + "\n"
        path = os.path.join(OUT_DIR, f"{slug}.md")
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(content)
        written += 1

    print(f"wrote {written} posts to {OUT_DIR}")


if __name__ == "__main__":
    main()
