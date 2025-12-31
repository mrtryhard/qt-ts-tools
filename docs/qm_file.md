Since the implementation is done through reverse engineering, some information may
be incorrect, incomplete, or working by pure luck. Reader (and users) be advised.

# Understanding QM File architecture

_Depending on the selected options in `lrelease`, the format may differ. `qt-ts-tools` chose to
support the default Qt Linguist output for now._

**Note**: QM File mostly use the big endian byte ordering.

## Layout of a QM File (version supported by qt-ts-tools)

### Blocks

Blocks are not magical, they are usually tagged with a single byte (`u8`) and the next 4 bytes (`u32`) determine the length of the block data.

```
+--------------------------------------------+
| [TAG]-[LEN-LEN-LEN-LEN]-[DATA-DATA...DATA] |
+--------------------------------------------+
```

### Overview

```
 +---------------------------------------------------------------------------+
 |                                  QmFile                                   |
 |                                                                           |
 |    +--------------+ +-----------+ +---------+ +---------+  +----------+   |
 |    | MagicNumber  | | Language  | | Hashes  | |Messages |  | Numerus  |   |
 |    |              | |           | |         | |         |  |          |   |
 |    +--------------+ +-----------+ +---------+ +---------+  +----------+   |
 |                                                                           |
 +---------------------------------------------------------------------------+
```

### Hash table usage

The hash table contain entry under the form of a list of pairs `{ Hash: u32, Offset: u32 }` where:

* `hash` corresponds to the original (untranslated) source string hashed with the [`elf hashing`](https://en.wikipedia.org/wiki/PJW_hash_function) algorithm
* `offset` corresponds to the position of the message **within** the messages table.

```
 +-------------------------------------+            
 |            Hashes Table             |            
 |     +----------------------------+  |            
 |     |           Entry            |  |            
 |     |                            |  |            
 |     |    +-------+ +-------+     |  |            
 |     |    | hash  | |offset |     |  |            
 |     |    | 4bytes| |4bytes |     |  |            
 |     |    +-------+ +-------+     |  |            
 |     |                  |         |  |            
 |     +------------------|---------+  |            
 |                        |            |            
 +------------------------|------------+            
                          |                         
                      points to                     
                          |                         
 +------------------------|------------------------+
 |                 Messages Table                  |
 |                        |                        |
 |                        v                        |
 |    +----------+  +----------+  +----------+     |
 |    |Message 1 |  |Message 2 |  |Message N |     |
 |    |          |  |          |  |          |     |
 |    +----------+  +----------+  +----------+     |
 |                                                 |
 +-------------------------------------------------+
```

## Message table structure

The messages table contains the source string (untranslated) and the translated string.
The source string is regular UTF-8 and my personal bet is because of the preprocessor on the C++ projects.
Then, the translated strings are in UTF-16 as it is what Qt uses by default. Context name is also UTF-8. 

```
 +---------------------------------------------------------------------------------+
 |                                 Messages Table                                  |
 |    +------------------------------------------------------------------------+   |
 |    |                                Message                                 |   |
 |    |                                                                        |   |
 |    |    +------------+ +-----------------++-----------+  +-------------+    |   |
 |    |    |Source utf8 | |Translated utf16 || Comments  |  |Context name |    |   |
 |    |    |            | |                 ||           |  |             |    |   |
 |    |    +------------+ +-----------------++-----------+  +-------------+    |   |
 |    |                                                                        |   |
 |    +------------------------------------------------------------------------+   |
 |                                                                                 |
 +---------------------------------------------------------------------------------+
```

## Testing / reverse engineering

All tests were done with `QtLinguist` and `ghex` for reverse engineering.
The numerus blocks in the `qm` files in the repository have their values replaced by zeroes since it is not yet supported.