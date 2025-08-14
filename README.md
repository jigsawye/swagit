![Swagit](https://i.imgur.com/kYSEMFD.png)

## Install

Swagit is written in Rust and distributed via crates.io.

```bash
cargo install swagit
```

## Usage

Once that's done, you can run this command inside your project's directory:

```bash
swagit
```

You can set an alias to use a more convenient command:

```bash
alias sg="swagit"
```

<img src="https://i.imgur.com/lZE5CG1.gif" width="500">

### Options

#### `--delete` or `-d`

Enter an interactive mode to select branches to be deleted.

<img src="https://i.imgur.com/8Vk1yqS.gif" width="800">

#### `--sync` or `-s`

Sync with remote and clean up merged branches. This command:
- Syncs current branch with remote (fast-forward only)
- Updates remote references
- Deletes merged branches automatically

## License

MIT © [Evan Ye](https://github.com/jigsawye)
