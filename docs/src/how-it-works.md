# How it works

`sess` doesn't reinvent terminal multiplexing — it uses tmux as its engine
and adds three things on top: a capture step, a storage format, and a
restore step. This section covers each.
