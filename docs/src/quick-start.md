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

To browse everything you've saved, just run `sess` with no arguments for
the interactive picker, which also shows how much disk space each session
is using.
