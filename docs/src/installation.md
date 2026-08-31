# Installation

## Requirements

- [tmux](https://github.com/tmux/tmux) installed and on your `PATH`.
- Linux (`sess` reads `/proc/<pid>/cmdline` to resolve the full command line
  running in each pane — see [How it works](./how-it-works/capture.md)).

## From a release binary

Download the latest tarball from the
[Releases page](https://github.com/sess-linux/sess/releases), then:

```bash
tar -xzf sess-VERSION-linux-x86_64.tar.gz
sudo mv sess /usr/local/bin/
```

## Via cargo / crates.io

If you already have Rust installed:

```bash
cargo install sess
```

## From source

```bash
git clone https://github.com/sess-linux/sess
cd sess
cargo build --release
sudo cp target/release/sess /usr/local/bin/
```

Verify it's on your `PATH`:

```bash
sess --version
```

## Optional: config file

Auto-save-by-default and environment variable persistence are both off
until you create `~/.config/sess/config.toml` — see
[Configuration](./configuration.md). Nothing here is required just to use
`sess`.
