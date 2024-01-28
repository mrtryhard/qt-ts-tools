# qt-ts-tools
This repository contains suite of tools for manipulating [Qt Framework](https://www.qt.io/product)'s [translation files](https://wiki.qt.io/QtInternationalization), contained in a single executable.

## Implemented functions
See `qt-ts-tools --help` for a list of operations in your version.

### Sort
```shell
./qt-ts-tools sort my_file.ts -o my_file_sorted.ts
```
## Limitations
* Sorting might not be 100% identical to Qt's. 
* Some tags may become self-closing (e.g. `<translation/>` instead of `<translation></translation>`)

## Planned work
In no particular order:

- [x] `sort`: outputs a sorted version of the provided file. 
- [ ] `diff`: outputs the differences between 2 translation files.
- [ ] `merge`: merge 2 translation files into a single output.
- [ ] `extract`: copy some specific elements via a filter (i.e. `unfinished` translations) to a new translation file.
- [ ] `strip`: remove some specific elements via a filter (i.e. `oldcomment` nodes) from the provided file.

## Philosophy
This tool aims to be simple to use and conservative in its decision. Therefore, no command shall modify the input file.
If an input file is modified without being explicitly asked, it is an undesirable behavior. 

## License
See [LICENSE](LICENSE).