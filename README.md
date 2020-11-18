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

**git-ignore** is released and maintained via [Homebrew](https://brew.sh). Technically, you can build from source or download the release binaries directly, but that is not recommended nor documented.

```
brew tap janniks/git-ignore
brew install git-ignore
```

<details>
<summary><i>Expand for uninstall instructions</i></summary>
<p><pre>brew untap janniks/git-ignore
brew uninstall git-ignore</pre></p>
</details>

## Usage 🚀

**git-ignore** is used to generate new `.gitignore` files or append content to existing ones.

From now on, you simply run `git ignore` to launch the interactive CLI, anytime you want to setup or change a `.gitignore` file.

## Why? ⚡️

...

## How? 💭

git-ignore is added as an external executable for git. Basically, if there are executables in your PATH that match `git-<command>` then they will become available through git as `git command`.

## Credits 🌎

- git-ignore uses templates from the [Toptal](https://www.toptal.com) [gitignore.io](https://gitignore.io) project.

## License ⚖️

[MIT](LICENSE)
