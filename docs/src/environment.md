# Environment variable persistence

By default, `sess` never captures environment variables — a snapshot's
`env` map is empty unless you explicitly opt in.

## Allow-listing variables

In [`~/.config/sess/config.toml`](./configuration.md):

```toml
[environment]
persist = ["NODE_ENV", "EDITOR", "PROJECT_ENV"]
```

On `sess save`, for every name in `persist` that's actually set in your
current shell, its value is captured into the snapshot. Names not in the
list are never touched, and names in the list that aren't currently set
are simply skipped.

## Secrets

`sess` isn't a secrets manager, and doesn't try to guess which of your
variables are sensitive on its own — it only ever looks at names you
explicitly listed. As a safety net, though, if an allow-listed name
contains a common secret-shaped substring —

```
*_KEY
*_TOKEN
*_SECRET
PASSWORD
AUTH
CREDENTIAL
```

— `sess save` still persists it (you asked for it by name), but prints a
warning so it doesn't happen silently:

```
warning: 'GITHUB_TOKEN' looks like it might hold a secret — persisting it
anyway because it's explicitly listed in your config.
```

## Restoring

On `sess open`, persisted variables are re-exported into every pane's
shell before its saved command is relaunched, so the command sees them the
same way it did originally. `sess` prints a one-line summary of what was
restored:

```
restoring 2 persisted env var(s): EDITOR, NODE_ENV
```

If a snapshot has no persisted variables — the common case — nothing is
printed and nothing changes.
