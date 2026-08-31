# Configuration

`sess` works with zero configuration. The config file only exists to turn
on optional behavior — auto-save by default, and environment variable
persistence — and lives at:

```
~/.config/sess/config.toml
```

It's entirely optional: a missing file, or one that fails to parse, just
means everything falls back to its defaults. `sess` never errors out
because of a config problem.

## Full example

```toml
[autosave]
enabled = true
interval = 30

[environment]
persist = ["NODE_ENV", "EDITOR", "PROJECT_ENV"]
```

## `[autosave]`

| Key        | Default | Meaning |
|------------|---------|---------|
| `enabled`  | `false` | If `true`, every `sess start <name>` turns on auto-save by default — no need for `--auto-save` each time. |
| `interval` | `30`    | Seconds between auto-saves. |

`sess start <name> --auto-save` / `--no-auto-save` always override this
per invocation, in either direction. See
[`start`](./commands/start.md) and [`auto-save`](./commands/auto-save.md).

## `[environment]`

| Key       | Default | Meaning |
|-----------|---------|---------|
| `persist` | `[]`    | Variable names allowed to be captured on `sess save`. Empty by default — nothing is ever persisted unless you list it here. |

See [Environment variable persistence](./environment.md) for the full
picture, including the secret-name warning.
