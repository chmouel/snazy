# snazy - a snazy json log viewer

Snazy is a simple tool to parse json logs and output them in a nice format with
nice colors.

As a [`tekton`](http://tekton.dev) developer this works pretty well with tekton
controllers and webhooks pods but the shoudld work as well with most knative
package and other pods using go-uber/zap.

## Screenshot

### Default

![screenshot](./.github/screenshot.png)

## Installation

### Binaries

Go to the [release](https://github.com/chmouel/snazy/releases) page and choose
your archive or package for your platform.

### Arch

You can install it [from aur](https://aur.archlinux.org/packages/snazy) with
your aurhelper, like yay :

```shell
yay -S snazy
```

### Homebrew

```shell
brew tap chmouel/snazy https://github.com/chmouel/snazy
brew install snazy
```

### Docker

```shell
kubectl logs deployment/pod foo|docker run -i ghcr.io/chmouel/gosmee
```

## Build from source

Snazy is using rust, if you want to compile it directly you just need to
checkout the source and run `cargo install`.

## Usage

You use `snazy` by "piping" logs into it :

```shell
kubectl logs deployment/controller|snazy
```

It supports streaming too, so if you do a `kubectl logs -f` it would just wait
for input.

If you need to stream a file you simply can do a `snazy < FILE`

If your input comes from <https://github.com/boz/kail> it will automatically
detect it and print the namespace/pod[container] :

![screenshot](./.github/screenshot-kail.png)

If you don't want to have the namespace/pod[container] printed you can add the
flag `--kail-no-prefix`.

If you want to highlight some pattern you can add the option `-r REGEXP` and
`snazy` will highlight it.

If you want to only show some levels, you can add the -f option with level
separated by commas, for example:

```shell
% kuibectl log pod|snazy -f warning,error
```

will only show warning and error fro the log.

You can customize the time printed with the `-t` option which respect the
[`strftime`](https://man7.org/linux/man-pages/man3/strftime.3.html) format
strings.

## FAQ

- I used to have a python script that does the same and more called
  (`[sugarjazy](https://github.com/chmouel/sugarjazy)`) but I wanted to
  experiment with Rust so I called this one
  `[snazy](https://www.urbandictionary.com/define.php?term=snazy)`.
- Why rust? Good question, it seems the kids like it but i still don't get it,
  maybe one day I will but it really take a few years to dig a programming
  language.

## Copyright

[Apache-2.0](./LICENSE)

## Authors

Chmouel Boudjnah <[@chmouel](https://twitter.com/chmouel)>
