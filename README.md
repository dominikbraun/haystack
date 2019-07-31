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
You can provide a whitelist of file extensions if you don't want all files to be searched.

```sh
$ haystack <path> <term> --ext rs go
```
To get a list of all available options, use `--help`.

```sh
$ haystack --help
```

> _haystack_ is still under development.
