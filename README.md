# Commit Message Editor

A small and simple tui editor to create and edit commit messages fast.

![A screen recording of cme in action using jj desc](./example.gif)

## Usage

Set `core.editor` to `cme` in your Git configuration:

```bash
git config --global core.editor "cme"
```

For jujutsu, set `ui.editor` to `cme` in your configuration:

```bash
jj config set --user ui.editor "cme"
```

## Configuration

Configuration is done via command-line arguments.
Notably, there is:

- `cme --fullscreen`: Opens the editor in fullscreen mode.
- `cme --title <title>`: Adds a title bar to the editor with the text `<title>`.
- `cme --height <lines>`: Sets the editor height to `<lines>` lines.

Run `cme --help` to see all the available options.  

In the example above, `cme --title "Enter commit description" --fullscreen` is used.

## Installation

This repo has a nix flake that can be used to install `cme`.

```bash
nix profile add github:guibi1/cme
```
