---
description: Manually create a manifest file for given mod.
---

Require a given path for mod folder, a link to search related info.

- Read spec of manifest from ![Manifest Proposal](https://raw.githubusercontent.com/martesi/spt-mod-manifest-proposal/refs/heads/main/manifest.json)
- Fetch info from given link.
- Try to fill all fields of the manifest, including icon, documentation.
- embbed the icon data as base64
- save documentation at "{mod_folder}/manifest/documentation.md"
- save manifest ab "{mod_folder}/manifest/manifest.json"