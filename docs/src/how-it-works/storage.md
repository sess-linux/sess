# Storage format

Each saved session is one JSON file at
`~/.local/share/sess/<name>.json` — no database, no binary format.

```json
{
  "version": 2,
  "name": "backend-debug",
  "created_at": "2026-08-30T03:16:25Z",
  "windows": [
    {
      "index": 0,
      "name": "dev",
      "layout": "b25f,80x24,0,0,2",
      "panes": [
        { "index": 0, "cwd": "/home/user/project", "command": "sleep 300" }
      ]
    }
  ],
  "env": { "NODE_ENV": "production" }
}
```

This is what `sess list` reports the size of — it's typically a few
kilobytes, since it stores layout and command lines, not process memory or
scrollback.

## Versioning and backward compatibility

`version` and `env` were both added in v0.3. Snapshots saved by earlier
versions of `sess` — without either field — still load without any
migration step: both default sensibly on read (`version` defaults to `1`,
`env` defaults to an empty map). If a future change to the format ever
needs an actual migration instead of a safe default, it'll bump
`CURRENT_SNAPSHOT_VERSION` and add an explicit conversion step rather than
break old snapshots outright.

## One reader for raw files

Every saved `.json` file is read through a single function
(`storage::list_all_raw`) that returns, per file, either the parsed
snapshot or the parse error as text — never silently skipping a broken
one. `sess list`, `sess prune`, and the
[state detection](../states.md) all build on top of this same read path
instead of re-parsing independently, so a corrupted file is reported the
same way everywhere it shows up.
