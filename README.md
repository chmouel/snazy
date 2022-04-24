# snazy - a snazy json log viewer

Snazy is a simple tool to parse json logs and output them in a nice format with
nice colors.

As a [`tekton`](http://tekton.dev) developer who has to dig into controller/webhook logs I wanted 
something that is a bit easier to look on the eyes and identify error/info/warning statements easily.

It's not only for `tekton` but would work well with projects using [`go-uber/zap`](https://github.com/uber-go/zap) library like [`knative`](https://knative.dev) and many others.

## Screenshot

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

If you need to parse a file you simply can use the shell with `snazy < FILE`

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
% kubectl log pod|snazy -f warning,error
```

will only show warning and error fro the log.

You can customize the time printed with the `-t` option which respect the
[`strftime`](https://man7.org/linux/man-pages/man3/strftime.3.html) format
strings.

## FAQ

- I have seen a tool like that before with another stupid name? I used to have a python script that does the same and more called
  ([sugarjazy](https://github.com/chmouel/sugarjazy)) but I wanted to experiment with Rust so I called this one
  [snazy](https://www.urbandictionary.com/define.php?term=snazy).
- Why rust? Good question, it seems the kids like it but i still don't get it,
  maybe one day I will but it really take a few years to dig a programming
  language.

## Copyright

[Apache-2.0](./LICENSE)

## Authors

Chmouel Boudjnah <[@chmouel](https://twitter.com/chmouel)>
