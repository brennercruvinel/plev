#!/usr/bin/env python3
"""Generate thumbnail screenshots for all demo projects using Playwright."""
import json
import os
import subprocess
import time
import signal
import sys
from playwright.sync_api import sync_playwright

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
THUMBS_DIR = os.path.join(BASE_DIR, "thumbs")
PORT = 9222

os.makedirs(THUMBS_DIR, exist_ok=True)

with open(os.path.join(BASE_DIR, "demos.json")) as f:
    demos = json.load(f)

# Start local server
server = subprocess.Popen(
    [sys.executable, "-m", "http.server", str(PORT)],
    cwd=BASE_DIR,
    stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,
)
time.sleep(1)

try:
    with sync_playwright() as p:
        browser = p.chromium.launch()
        page = browser.new_page(viewport={"width": 1280, "height": 720})

        for i, d in enumerate(demos):
            slug = d["dir"].replace("/", "-")
            out = os.path.join(THUMBS_DIR, f"{slug}.jpg")

            if os.path.exists(out):
                print(f"[{i+1}/{len(demos)}] SKIP {d['name']}")
                continue

            path = d["path"]
            if path == ".":
                url = f"http://localhost:{PORT}/{d['dir']}/index.html"
            else:
                url = f"http://localhost:{PORT}/{d['dir']}/{path}/index.html"

            print(f"[{i+1}/{len(demos)}] {d['name']} -> {url}")
            try:
                page.goto(url, timeout=12000, wait_until="load")
                page.wait_for_timeout(3000)
                page.screenshot(path=out, type="jpeg", quality=75)
                print(f"  OK -> {slug}.webp")
            except Exception as e:
                print(f"  ERRO: {e}")
                # Save a blank placeholder
                try:
                    page.goto("about:blank")
                    page.wait_for_timeout(200)
                except:
                    pass

        browser.close()
finally:
    server.terminate()
    server.wait()

print(f"\nDone! Thumbnails in {THUMBS_DIR}/")
print(f"Total files: {len(os.listdir(THUMBS_DIR))}")
