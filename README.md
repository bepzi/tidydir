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

### Additional Integration

`tidydir` won't be of much use if it's not being run; therefore, it's
best to pair it with scripts that can call `tidydir` regularly and
notify you when files and folders have become stale.

On my machine I refresh the list of files in my `Downloads` folder
every time I log in. Then, I send a notification if invoking `tidydir`
results in any output.

``` bash
# ~/.bash_profile, sourced only once (each time I log in)
tidydir track ~/Downloads/*
if [ $(tidydir | wc -c) -gt 0 ]; then
    notify-send "tidydir" "$(tidydir)"
fi
```

## Contributing

...is welcomed! Please submit any issues and pull requests, though all
code should be formatted with `cargo fmt`, first.
