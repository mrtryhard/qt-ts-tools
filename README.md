# qt-ts-tools
![Build](https://github.com/mrtryhard/qt-ts-tools/actions/workflows/rust.yml/badge.svg) ![Build](https://github.com/mrtryhard/qt-ts-tools/actions/workflows/security.yml/badge.svg)

This repository contains suite of tools for manipulating [Qt Framework](https://www.qt.io/product)'s [translation files](https://wiki.qt.io/QtInternationalization), contained in a single executable.

## Implemented functions
See `qt-ts-tools --help` for a list of operations in your version.

```shell
# Sort
./qt-ts-tools sort my_file.ts -o my_file_sorted.ts
# Strip symbols
./qt-ts-tools strip my_file.ts -t vanished -o my_file_stripped.ts
# Merge translation files
./qt-ts-tools merge base.ts changes.ts -o merged_file.ts 
# Extract only specific type of translation
./qt-ts-tools extract my_file.ts -t obsolete -o extracted.ts
# Print the summary of the translation file
./qt-ts-tools stat my_file.ts
```

## Limitations
* The output format may change a little bit i.e. self-closing tags becomes full tags
* QtLinguist full functionality might not be fully replicated

## Philosophy
This tool aims to be simple to use and conservative in its decision. Therefore, no command shall modify the input file.
If an input file is modified without being explicitly asked, it is an undesirable behavior. 

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution
See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed information about contributing.

### Bugs report or feature request
You may report any bug reports or feature requests through [Github's issue tracker](https://github.com/mrtryhard/qt-ts-tools/issues/).

### Code contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
