# tidydir

Keeps files from getting stale on your hard drive. 🧹🗑️

`tidydir` is a simple CLI application that can keep track of files and
folders on your hard drive so that you know when they've been sitting
around for too long. No more cluttered `Downloads` folders!

## Usage

List all tracked files which have become "stale" (default: 2 days):

``` bash
tidydir
```

List all tracked files that have been sitting around for more than 3
hours:

``` bash
tidydir -s 10800  # 3 hours, in seconds
```

Register all files and folders within `~/Documents`:

``` bash
tidydir track ~/Documents/*
```

Unregister all files and folders within `~/Documents`:

``` bash
tidydir untrack ~/Documents/*
```

Unregister all files and folders, everywhere:

``` bash
tidydir untrack --all
```

List all files and folders currently tracked by `tidydir`:

``` bash
tidydir list
```

## Contributing

...is welcomed! Please submit any issues and pull requests, though all
code should be formatted with `cargo fmt`, first.
