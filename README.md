![License](https://img.shields.io/github/license/fossable/autovet)
![build](https://github.com/fossable/autovet/actions/workflows/build.yml/badge.svg)
![Lines of code](https://img.shields.io/tokei/lines/github/fossable/autovet)
![Stars](https://img.shields.io/github/stars/fossable/autovet?style=social)

**autovet** continuously searches for security breaches in open source libraries and applications.

<hr>

## Recently processed packages

| package | version | channel  | last check  | syscall coverage | result   |
|---------|---------|----------|-------------|------------------|----------|
|         |         |          |             |                  |          |

<hr>

## Motivation

Every time I update my Arch Linux systems, I'm impressed at the quantity of
packages that have changed:

```
$ sudo pacman -Syu
:: Synchronizing package databases...
 core is up to date
 extra is up to date
 community is up to date
:: Starting full system upgrade...
resolving dependencies...
looking for conflicting packages...

Packages (48) cmake-3.24.0-1  libva-2.15.0-2  meson-0.63.1-2  python-3.10.6-1  rubygems-3.3.19-1  seabios-1.16.0-3 ...

Total Download Size:    63.81 MiB
Total Installed Size:  283.88 MiB
Net Upgrade Size:        2.81 MiB

:: Proceed with installation? [Y/n]
```

The question that should occur to any security-conscious person is: _how can
I trust all of these updates coming from thousands of developers around the world?_

Sure, all updates are cryptographically signed to prove authenticity, but remember how
that thwarted the SolarWinds attack? Yeah, it didn't.

And what if a developer goes rogue and decides to "cash out" on the popularity of their open-source
project by sneaking a cryptocurrency wallet uploader into their next release? They'll definitely be
caught, but maybe not before significant damage is done.

What we need is an automated first line of defense that can provide some shred of confidence
that installing the latest version of `supertuxcart` isn't going to spawn a bitcoin
miner in the background. Of course it'll never be possible to reach 100% confidence
because security doesn't work like that, but something is better than nothing.

So that's exactly what **autovet** is designed to do. The best part is, you don't
have to do anything to use it! **autovet** automatically processes packages as soon
as they are released and creates issues on the appropriate repositories.

#### Why rebuilderd isn't enough

[rebuilderd](https://github.com/kpcyrd/rebuilderd) is a project that attempts to recreate official release artifacts from source
to prove there were no unwelcome additions in the build process.

This is a valuable tool, but it can't catch compromises to the source code itself.

## Dynamic Application Security Testing (DAST)

**autovet** runs programs in virtual machines of varying configurations and just watches
what happens. It's not possible to find every compromise this way, but it certainly
makes it more difficult for an attacker to slip in malicious code unnoticed.

### System Calls

The most powerful indicator that **autovet** considers is the series of system calls that a
program invokes during execution. (For this reason, **autovet** isn't effective on kernel
modules).

For example, if an update to `jq` is somehow compromised and starts trying to open
network sockets with `connect()`, we know something is very wrong. This kind of detection
is possible because **autovet** knows approximately what system calls to expect according
to the previous version of the program.

How do we know the previous version of the program isn't compromised as well? Currently it's just
a manual step that happens when a package is first initialized. Someone has to look over the
system call list to make sure it's reasonable for the particular program.

### Evasion Tactics

Malware often tries to detect when it's under scrutiny and appear benign. **autovet**
attempts to detect that detection and flags programs that seem to evade analysis.

