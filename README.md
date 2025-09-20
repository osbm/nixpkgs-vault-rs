# nixpkgs-vault

I want to generate an obsidian vault from a nixpkgs git revision.

It will have all the packages as notes, and the dependencies between them as links. Each note will contain the package's metadata, such as its name, version, and description. Additionally, I want to include some statistics about the nixpkgs repository, such as the number of packages, the number of maintainers, and the number of packages per maintainer.

# expected cli usage

```
$ nixpkgs-vault --revision <git-revision> --output <output-directory>
```

or

```
$ nixpkgs-vault -r <git-revision> -o <output-directory>
```

or

```
$ nixpkgs-vault --tag <git-tag> -o <output-directory>
```

or

```
$ nixpkgs-vault -t <git-tag> -o <output-directory>
```

or

```
$ nixpkgs-vault
```
defaults to the latest nixpkgs-unstable revision and outputs to ./nixpkgs-vault


# Problem number 1: get the nixpkgs

