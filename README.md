<h1 align="center" style="font-weight: bold !important">git-ignore 🙈</h1>

<p align="center">
  <strong>git-ignore</strong> lets you generate <code>.gitignore</code> files for your repositories right from the terminal
</p>

<p align="center">
  <a href="https://news.ycombinator.com/item?id=25148371">
    <img alt="Featured on Hacker News" src="https://hackerbadge.now.sh/api?id=25148371&type=orange" width="134" height="34"/>
  </a>
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

**git-ignore** is released and maintained via [Homebrew](https://brew.sh). Technically, you can build from source or download the release binaries directly, however we have not had time to add those instructions yet.

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

## Usage 🚀

**git-ignore** is used to generate new `.gitignore` files or append content to existing ones.

From now on, you simply run `git ignore` to launch the interactive CLI, anytime you want to setup or change a `.gitignore` file.

## Why? ⚡️

Every time I start a new project/repository, I need a `.gitignore` file. And every time I missed something that had to be added later—often after unstashing/reverting, because of those pesky `git add -all` I love so dearly. Then came [gitignore.io](https://gitignore.io) and made my life a lot easier. Sadly, not too long ago, Toptal decided to rebrand the site a bit (not too much, but we developers are purists). So, the next time I spun up a new repository, I started procrastinating. I no longer wanted to have to leave the terminal to setup a .gitignore file. And thus **git-ignore** was born. You can now utilize battle-tested ignore templates right from your terminal.

## How? 💭

git-ignore is added as an external executable for git. Basically, if there are executables in your PATH that match `git-<command>` then they will become available through git as `git command`.

git-ignore uses GitHub and Toptal APIs to fetch the ignore templates.

## Credits 🌎

- git-ignore uses templates from the [Toptal](https://www.toptal.com) [gitignore.io](https://gitignore.io) project.

## License ⚖️

[MIT](LICENSE)
