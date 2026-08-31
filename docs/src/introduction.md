# Introduction

**sess** saves the state of a terminal session — pane layout, working
directories, and the commands that were running — under a name, and lets
you reopen it later, even after a reboot.

Tools like `tmux-resurrect` or `sesh` already solve part of this, but they
assume you're already living in the tmux plugin ecosystem (TPM and
friends). `sess` is a single standalone binary: it uses tmux as its
underlying engine, but you never have to touch tmux plugins to use it.

```bash
sess start backend-debug   # start a new tracked session
# ... work normally ...
sess save                  # save it under a name
# ... close the terminal, reboot, come back tomorrow ...
sess open backend-debug    # restored, panes and commands back where they were
```

Since v0.3, sess also tracks a **state** for every session (running,
saved, stale, or broken — see [Session states](./states.md)), can
**auto-save** in the background, optionally persist a handful of
**environment variables**, and check its own health with `sess doctor`.

This book covers installation, session states, configuration, every
command, and the internals: how a session is captured, how it's restored,
and the on-disk storage format.
