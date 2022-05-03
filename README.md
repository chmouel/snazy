[![Version](https://img.shields.io/crates/v/snazy.svg)](https://crates.io/crates/snazy) [![AUR](https://img.shields.io/aur/version/snazy-bin)](https://aur.archlinux.org/packages/snazy-bin) [![CICD](https://github.com/chmouel/snazy/actions/workflows/rust.yaml/badge.svg)](https://github.com/chmouel/snazy/actions/workflows/rust.yaml) [![pre-commit](https://img.shields.io/badge/pre--commit-enabled-brightgreen?logo=pre-commit&logoColor=white)](https://github.com/pre-commit/pre-commit)

# snazy - a snazzy json log viewer

Snazy is a simple tool to parse json logs and output them in a nice format with
nice colors.

As a [`tekton`](http://tekton.dev) developer who has to dig into controller/webhook logs I wanted
something that is a bit easier to look in the eyes and identify error/info/warning statements easily.

You do not have to use it only with `tekton` but work well with projects that uses
[`go-uber/zap`](https://github.com/uber-go/zap) library like
[`knative`](https://knative.dev) and many others.

## Screenshot

![screenshot](./.github/screenshot.png)

## Installation

### [Binaries](https://github.com/chmouel/snazy/releases)

Go to the [release](https://github.com/chmouel/snazy/releases) page and grab
the archive or package targeting your platform.

### [Arch](https://aur.archlinux.org/packages/snazy-bin)

With your favourite aurhelper for example [yay](https://github.com/Jguer/yay) :

```shell
yay -S snazy-bin
```

### [Nix/NixOS](https://nixos.org/)

This repository includes a `flake` (see [NixOS Wiki on
Flakes](https://nixos.wiki/wiki/Flakes)).

If you have the `nix flake` command enabled (currenty on
nixos-unstable, `nixos-version` >= 22.05)

```shell
nix run github:chmouel/snazy -- --help # your args are here
```

You can also use it to test and develop the source code:

```shell
nix develop # drops you in a shell with all the thing needed
nix flake check # runs cargo test, rustfmt, …
```

### [Homebrew](https://homebrew.sh)

```shell
brew tap chmouel/snazy https://github.com/chmouel/snazy
brew install snazy
```

### [Crates.io](https://crates.io/crates/snazy)

```shell
cargo install snazy
```

### [Docker](https://github.com/chmouel/snazy/pkgs/container/snazy)

```shell
kubectl logs deployment/pod foo|docker run -i ghcr.io/chmouel/gosmee
```

## Build from [source](https://github.com/chmouel/snazy)

Snazy is build with rust, if you want to compile it directly you just need to
grab the source and run `cargo build`.

## Usage

* Usually you use `snazy` by "piping" logs into it :

```shell
kubectl logs deployment/controller|snazy
```

* It supports streaming too. When you have a `kubectl logs -f` it will just wait
for input and snazzily print your logs from the stream (one line at a time).

* you can pass one or many files on the command line to `snazy` and it will
  parse them rather than using the standard input.

* If you do not pass a file and your input comes from
<https://github.com/boz/kail> it will automatically detect it and print the
`namespace/pod[container]` as prefix :

![screenshot](./.github/screenshot-kail.png)

* If you want to customize the kail format, you can do it with the flag
  `--kail-prefix-format` it will replace the variable `{namespace} {pod}
  {container}` by its value. If you for example only want to print the `pod` you can do :

     `--kail-prefix-format "{pod}"`

  or set the environement variable `SNAZY_KAIL_PREFIX_FORMAT` to make it permanent.

* If you do not any prefix for kail you can pass the `--kail-no-prefix` flag.

* If you want to highlight some patterns you can add the option `-r REGEXP` and
`snazy` will highlight it. You can have many `-r` switches with many
regexps, and you get different highlight for each match.

* If `snazy` don't recognize the line as json it will symply straight print
  it. Either way it will still apply regexp highlighting of the `-r` option or
  do the action commands matching (see below). This let you use it for any logs
  to do some regexp highlighting and action on pattern.

* If you want to only show some levels, you can add the flag `-f` to filter by
  level or many `-f` for many levels, for example, this only show warning and
  error from the log:

```shell
% kubectl log pod|snazy -f warning -f error
```

* If you pass the flag `--level-symbols` or set the environment variable `SNAZY_LEVEL_SYMBOLS`, snazy will show some pretty emojis rather than plain log level label :

![snazy level symbols](.github/screenshot-level-symbols.png)

* You can customize the time printed with the `-t` flag (or the environment
variable `SNAZY_TIME_FORMAT`), the variable respect the UNIX
[`strftime`](https://man7.org/linux/man-pages/man3/strftime.3.html) format
strings.

* You can do your own field matching with the `-k/--json-keys` flag, it accepts the
field `msg`, `level` and `ts`. Those fields target a key in json used for
parsing. The values should be:

  * `msg`: The message text (string)
  * `level`: The log level (eg: info) (string)
  * `ts`: The timestamp, a float or a datetime

  If any of those fields are missing the parser will fails.

* Snazy support action command on regexp, which mean if you have a regexp
  matching a message it will run an action on it. It currently support only one
  action one regexp. If you specify the string `"{}"` it will be expanded to
  the matched string. For example on macOS this command will display a
  notification with the pipelinerun that has succeeded:

  ```shell
  snazy --action-regexp "pipelinerun(s)?\s*.*has success" --action-command "osascript -e 'display notification \"{}\"'"

## Shell completions

Shell completions are available for most shells in the [misc/completions](./misc/completions) and it will be automatically installed with the aur/brew package.

## FAQ

* I have seen a tool like that before with another stupid name? I used to have a python script that does the same and more called
  ([sugarjazy](https://github.com/chmouel/sugarjazy)) but I wanted to experiment with Rust, so I called this one
  [snazy](https://www.urbandictionary.com/define.php?term=snazy).
* You missed a z to snazzy. True that. But snazy is easier to google than snazzy :p
* Why rust? Good question, it seems the kids like it, but I still don't get it,
  maybe one day I will, but it really takes a few years to dig a programming
  language.
* I have seen some of this code already 🤨, yep being a noob I much digged
  into the source code of (the super-duper great)
  [sharkdp/fd](https://github.com/sharkdp/fd)

## Copyright

[Apache-2.0](./LICENSE)

## Authors

Chmouel Boudjnah <[@chmouel](https://twitter.com/chmouel)>
