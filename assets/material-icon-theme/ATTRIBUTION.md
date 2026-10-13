# Material Icon Theme assets

This directory contains the icon assets from [Material Icon Theme](https://github.com/material-extensions/vscode-material-icon-theme), version 5.39.0, released under the MIT License. The corresponding license text is in `LICENSE`.

The original upstream SVG files are in `icons/`. PNG versions in `png/` were rasterized from those SVGs at 48 × 48 pixels for Sublemon's embedded project-tree textures. The small set of upstream association names without standalone SVG files has matching variants derived from the nearest base icon in the same pack.

`associations.tsv` maps upstream filename, extension, and folder-name associations to icon names. It was generated from the upstream `src/core/icons/fileIcons.ts` and `folderIcons.ts` manifests by `scripts/generate_icon_associations.py`.
