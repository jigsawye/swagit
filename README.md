![Swagit](https://i.imgur.com/kYSEMFD.png)

## Install

Since version 1.0.0, swagit has been rewritten in Rust, so you can install it directly via cargo.

```bash
cargo install swagit
```

### npm（legacy）

The npm version wraps the binary with JavaScript, so it will be slower than the crates version.

```bash
npm install -g swagit
pnpm add -g swagit
yarn global add swagit
```

## Usage

Once that's done, you can run this command inside your project's directory:

```bash
swagit
# shortcut
sg
```

<img src="https://i.imgur.com/lZE5CG1.gif" width="500">

### Options

#### `--delete` or `-d`

Enter an interactive mode to select branches to be deleted.

<img src="https://i.imgur.com/8Vk1yqS.gif" width="800">

#### `--sync` or `-s` (alpha test)

Sync all branches and delete merged branches.

## License

MIT © [Evan Ye](https://github.com/jigsawye)
