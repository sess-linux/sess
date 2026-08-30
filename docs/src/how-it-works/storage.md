# Storage format

Each saved session is one JSON file at
`~/.local/share/sess/<name>.json` — no database, no binary format.

```json
{
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
  ]
}
```

This is what `sess list` reports the size of — it's typically a few
kilobytes, since it stores layout and command lines, not process memory or
scrollback.
