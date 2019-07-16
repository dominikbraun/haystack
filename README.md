<h1 align="center">
	<br>
	<img src="https://sternentstehung.de/haystack-readme.png" alt="haystack">
	<br>
	<br>
</h1>

> A fast & simple text search across files.

## Basic Usage

You have to provide at least a directory path to search in and a search term.

```sh
$ haystack <path> <term>
```

_haystack_ searches case-sensitive by default. However, you can opt-in to case-insentive search.

```sh
$ haystack <path> <term> --case-insensitive
```
To get a list of all available options, use `--help`.

```sh
$ haystack --help
```

> _haystack_ is still under development.
