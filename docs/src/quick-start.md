# Quick start

Start a new session tracked by `sess`:

```bash
sess start backend-debug
```

This drops you into a fresh tmux session named `backend-debug`. Work
normally — open editors, run servers, split panes, whatever you need.

When you're ready to step away, save it:

```bash
sess save
```

By default this saves under the current tmux session's name. Pass a name
explicitly to save it under something else:

```bash
sess save backend-debug-friday
```

Later — tomorrow, after a reboot, whenever — reopen it:

```bash
sess open backend-debug
```

`sess` recreates the pane layout exactly, `cd`s each pane back to where it
was, and relaunches whatever was running.

## Switching between sessions

To browse everything you've saved, run `sess switch` (or just `sess` with
no arguments — they do the same thing):

```bash
sess switch
```

This opens an interactive picker showing every session's
[state](./states.md), pane count, and how much space it's using. Selecting
one behaves according to its state: it attaches if it's already running,
restores it if it's only saved, asks before restoring something
[stale](./states.md), and shows you what's wrong if it's
[broken](./states.md).

## Closing without losing your snapshot

```bash
sess close backend-debug
```

Kills the live tmux session but leaves the saved snapshot untouched —
useful when you want to free up the running session without discarding
what you saved.

## Never forgetting to save

```bash
sess start backend-debug --auto-save
```

Starts a background loop that periodically re-saves the session on its
own. See [Configuration](./configuration.md) to make this the default for
every new session, and the [`auto-save`](./commands/auto-save.md) command
page for the details.
