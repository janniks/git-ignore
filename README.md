<h1 align="center" style="font-weight: bold !important">git-ignore 🙈</h1>

<p align="center">
  <strong>git-ignore</strong> lets you generate <code>.gitignore</code> files for your repositories right from the terminal
</p>

<h3 align="center">
  <a href="#installation-">Installation</a>
  <span> · </span>
  <a href="#usage-">Usage</a>
  <span> · </span>
  <a href="#why-%EF%B8%8F">Why?</a>
  <span> · </span>
  <a href="#how-">How?</a>
  <span> · </span>
  <a href="#credits-">Credits</a>
  <span> · </span>
  <a href="#license-%EF%B8%8F">License</a>
</h3>

<p align="center">
  <img alt="Example usage" src="docs/images/example.gif" style="border-radius:3px">
</p>

## Installation 🛠

### macOS (via [Homebrew](https://brew.sh))

**git-ignore** is released and maintained via [Homebrew](https://brew.sh), which needs to be installed first.

Run the following commands to install:

```
brew tap janniks/git-ignore
brew install git-ignore
```

<details>
<summary><i>Expand for uninstall instructions</i></summary>
<p><pre>brew untap janniks/git-ignore
brew uninstall git-ignore</pre></p>
</details>

### Build from source (via Cargo)

> Tested on: _macOS, Archlinux_

1. Install [Rust](https://www.rust-lang.org) and Cargo (e.g. via [rustup](https://rustup.rs)).
2. Clone the repository via git and enter the project folder:

```
git clone https://github.com/janniks/git-ignore.git
cd git-ignore
```

3. Run Cargo's `build` command:

```
cargo build
```

> The binary is generated into the `target/debug` directory.
> If you want to run the command globally, you need to move it to a directory covered by your PATH environment variable (e.g. on Unix systems to the `/usr/local/bin` directory).
>
> If `~/.cargo/bin/` is already in your environment's PATH, your can run `cargo install --path .` to build and move the executable there (and skip step 4).

4. Move the executable:

```
mv target/debug/git-ignore /usr/local/bin/git-ignore
```

<details>
<summary><i>Expand for uninstall instructions</i></summary>
<p>Simply delete the executable from wherever it was moved:<pre>rf /usr/local/bin/git-ignore</pre><pre>rf ~/.cargo/bin/git-ignore</pre><pre>cargo uninstall git-ignore</pre></p>
</details>

## Usage 🚀

**git-ignore** is used to generate new `.gitignore` files or append content to existing ones.

From now on, you simply run `git ignore` to launch the interactive CLI, anytime you want to setup or change a `.gitignore` file.

## Why? ⚡️

Every time I start a new project/repository, I need a `.gitignore` file. And every time I missed something that had to be added later—often after unstashing/reverting, because of those pesky `git add -all` I love so dearly. Then came [gitignore.io](https://gitignore.io) and made my life a lot easier. Sadly, not too long ago, Toptal decided to rebrand the site a bit (not too much, but we developers are purists). So, the next time I spun up a new repository, I started procrastinating. I no longer wanted to have to leave the terminal to setup a .gitignore file. And thus **git-ignore** was born. You can now utilize battle-tested ignore templates right from your terminal.

## How? 💭

git-ignore is added as an external executable for git. Basically, if there are executables in your PATH that match `git-<command>` then they will become available through git as `git <command>`.

git-ignore uses GitHub and Toptal APIs to fetch the ignore templates.

## Credits 🌎

- git-ignore uses templates from the [Toptal](https://www.toptal.com) [gitignore.io](https://gitignore.io) project.

## License ⚖️

[MIT](LICENSE)
