<h1 align="center">
	<br>
	<img src="https://sternentstehung.de/haystack-dark-readme.png" alt="haystack">
	<br>
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
_haystack_ searches for all files it can get. You can provide a whitelist of file extensions to filter only for specific ones.
This is NOT case sensitive

```sh
$ haystack <path> <term> --whitelist rs go
```
To get a list of all available options, use `--help`.

```sh
$ haystack --help
```

> _haystack_ is still under development.
