---
tags: [shell, dev]
title: Backup and Setup a Mac from the Terminal
link: https://gist.github.com/dustinknopoff/f110d6a247a3881d49bba6ef125997d2
date: 2019-05-26 08:31
---

Setting up a computer is not the most enjoyable process. Having to copy over files, reinstall applications, and get system settings exactly as they were before is tedious and time consuming. On Macs, Time Machine exists as an option to essentially clone your previous state. However, I prefer to use a new computer as a kind of spring cleaning, only passing over essentials and not any cruft in the weeds of `/Library`.

There are 3 main tools I use for backing up/making the setup process simple:

1. `Brewfile`/Homebrew
2. `mackup`
3. Bash

## Homebrew

Homebrew is a package manager for the mac. It allows you easily install terminal applications with a single command. In addition, I use 3 branches of Homebrew, `brew cask`, `brew cask-fonts` and `mas`.

Where `brew` is for installing terminal tools, `brew cask` is for installing GUI applications. Similarly, `brew cask-fonts` is for installing fonts. `mas` is slightly different as it's a terminal wrapper around the mac app store for installing/updating applications.

If you don't have `brew` yet, enter the following in to your terminal:

```bash
/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
```

## Brewfile

`Brewfile` is essentially a file that keeps track of everything you've installed through homebrew.

```bash
brew "fd"
brew "fdupes"
brew "ffmpeg"
brew "findutils"
brew "fish"
brew "flow"
brew "fswatch"
brew "fzf"
brew "fzy"
cask "transmission"
cask "vagrant"
cask "visual-studio-code"
cask "vivaldi"
cask "vlc"
cask "webstorm"
cask "whatsapp"
mas "Unsplash Wallpapers", id: 1284863847
mas "Xcode", id: 497799835
cask_args appdir: "/Applications"
```
> a small example `Brewfile`

To setup a `Brewfile` with your existing installed packages,

```bash
brew bundle dump
```

[Check out the repo for more information](https://github.com/Homebrew/homebrew-bundle)

## Mackup

A tool that near automatically backs up configuration settings for [many](https://github.com/lra/mackup#supported-applications) applications.

Installed with `brew install mackup` and set to backup to iCloud, Dropbox, Git, or more. [follow directions here to specify](https://github.com/lra/mackup/blob/master/doc/README.md) *I use iCloud*

## Pulling it all together with scripts!

I have 2 main scripts, one that I run once a week to keep an mostly update version of backups and one for restoring.

## Backing up

It's pretty simple!

```bash
echo "Brewfile"
cd ~
brew bundle dump --force
echo 'cask_args appdir: "/Applications"' >> ~/Brewfile
bat ~/Brewfile > ~/Documents/dotfiles/init/Brewfile

mackup backup
```

That's it!

The `dotfiles` folder is a git repository and `cask_args appdir...` tells homebrew to always install applications to the applications folder. It is not included here but this script (called `backup.sh` from now on) also includes other little commands like using `rsync` to backup my `Documents` folder to iCloud (I use the same script to backup to my external hard drives as well). It also includes a git commit and push of the changed Brewfile.

## Restoring

The restoring is slightly more complicated if only because it includes the installation of git, homebrew, and `mas`. 

```bash
xcode-select --install

/usr/bin/ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
brew update
brew upgrade
brew install mas
```

> **NOTE** `mas` requires logging in to the actual Mac App Store to work

```bash
brew bundle ./init/Brewfile
brew cleanup

mackup restore 
```

I've named this file `restore.sh`. This is also in my `dotfiles` repository. Restoring terminal tools, applications, fonts, and application configurations is then as simple as downloading the `dotfiles` repo and running `sh restore.sh`.
