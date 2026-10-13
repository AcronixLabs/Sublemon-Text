#!/usr/bin/env python3
"""Extract file/folder associations from Material Icon Theme's TypeScript manifests."""
from pathlib import Path
import re
import argparse

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('upstream_icons_source', type=Path, help='path to the pack source src/core/icons directory')
SOURCE = parser.parse_args().upstream_icons_source


def matching(source: str, start: int, opening: str, closing: str) -> int:
    depth = 0
    quote = None
    escaped = False
    for index in range(start, len(source)):
        char = source[index]
        if quote:
            if escaped:
                escaped = False
            elif char == '\\':
                escaped = True
            elif char == quote:
                quote = None
            continue
        if char in "'\"`":
            quote = char
        elif char == opening:
            depth += 1
        elif char == closing:
            depth -= 1
            if depth == 0:
                return index
    raise ValueError(f'unclosed {opening}')


def array(source: str, key: str) -> list[str]:
    match = re.search(rf'\b{key}\s*:\s*\[', source)
    if not match:
        return []
    start = match.end() - 1
    body = source[start + 1:matching(source, start, '[', ']')]
    return re.findall(r"['\"]([^'\"]+)['\"]", body)


def objects(source: str, start: int, end: int):
    depth = 0
    quote = None
    escaped = False
    begin = None
    for index in range(start, end):
        char = source[index]
        if quote:
            if escaped:
                escaped = False
            elif char == '\\':
                escaped = True
            elif char == quote:
                quote = None
            continue
        if char in "'\"`":
            quote = char
        elif char == '{':
            if depth == 0:
                begin = index
            depth += 1
        elif char == '}':
            depth -= 1
            if depth == 0 and begin is not None:
                yield source[begin:index + 1]
                begin = None


def manifest_objects(source: str, marker: str):
    match = re.search(marker, source)
    if not match:
        raise ValueError(f'missing manifest marker {marker}')
    start = source.index('[', match.start())
    end = matching(source, start, '[', ']')
    return objects(source, start + 1, end)


file_source = (SOURCE / 'fileIcons.ts').read_text()
file_rows = []
for obj in manifest_objects(file_source, r'icons:\s*parseByPattern\s*\('):
    name = re.search(r"\bname\s*:\s*['\"]([^'\"]+)['\"]", obj)
    if not name:
        continue
    icon = name.group(1)
    for extension in array(obj, 'fileExtensions'):
        file_rows.append(('ext', extension.lower(), icon))
    for filename in array(obj, 'fileNames'):
        file_rows.append(('name', filename.replace('\\', '/').lower(), icon))

folder_source = (SOURCE / 'folderIcons.ts').read_text()
folder_rows = []
# Use the first theme, named "specific", which contains all specific folder mappings.
for obj in manifest_objects(folder_source, r'icons:\s*\['):
    name = re.search(r"\bname\s*:\s*['\"]([^'\"]+)['\"]", obj)
    if not name:
        continue
    for folder in array(obj, 'folderNames'):
        folder_rows.append((folder.lower(), name.group(1)))

# Stable first occurrence wins where the upstream theme intentionally aliases patterns.
seen = set()
lines = ['# kind<TAB>pattern<TAB>icon']
for row in file_rows:
    if row not in seen:
        lines.append('\t'.join(row))
        seen.add(row)
for folder, icon in folder_rows:
    row = ('folder', folder, icon)
    if row not in seen:
        lines.append('\t'.join(row))
        seen.add(row)
(ROOT / 'assets/material-icon-theme/associations.tsv').write_text('\n'.join(lines) + '\n')
print(f'wrote {len(seen)} file and folder associations')
