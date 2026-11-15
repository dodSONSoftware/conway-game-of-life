#!/usr/bin/env python3
"""Download authentic ConwayLife RLE assets from the public pattern directory.

Run from the project root: python3 scripts/fetch_patterns.py
Only patterns found in the official index are downloaded. Invalid or missing
patterns remain absent from the in-game library; no synthetic substitutes.
"""
import html.parser
import json
from pathlib import Path
import urllib.request
import urllib.error

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets" / "patterns"
BASE = "https://conwaylife.com/patterns/"

class IndexParser(html.parser.HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = set()
    def handle_starttag(self, tag, attrs):
        if tag == "a":
            for key, value in attrs:
                if key == "href" and value:
                    self.links.add(value.split("/")[-1])

def fetch(url, limit=10_000_000):
    with urllib.request.urlopen(urllib.request.Request(url, headers={"User-Agent":"GameOfLifePatternImporter/1.0"}), timeout=30) as resp:
        data = resp.read(limit + 1)
    if len(data) > limit:
        raise ValueError("Pattern exceeds configured download limit")
    return data

def main():
    catalog = json.loads((ASSETS / "catalog.json").read_text())
    parser = IndexParser()
    parser.feed(fetch(BASE, 2_000_000).decode("utf-8", "replace"))
    success = 0
    for item in catalog:
        filename = item["file"]
        target = ASSETS / filename
        if filename not in parser.links:
            print(f"UNAVAILABLE {item['name']}: {filename} absent from directory")
            continue
        try:
            raw = fetch(BASE + filename)
            txt = raw.decode("utf-8")
            if filename.endswith(".mc"):
                valid = txt.lstrip().startswith("[M2]")
            else:
                valid = "x =" in txt and "!" in txt
            if not valid:
                raise ValueError("Invalid or unsupported RLE/Macrocell format")
            target.write_bytes(raw)
            success += 1
            print(f"OK {item['name']}: {len(raw):,} bytes")
        except (OSError, UnicodeError, ValueError) as exc:
            print(f"FAILED {item['name']}: {exc}")
    print(f"Installed {success}/{len(catalog)} patterns. Restart Game of Life to load them.")

if __name__ == "__main__":
    main()
