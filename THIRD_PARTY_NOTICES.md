# Third-party notices

## UnRAR

The installer links the UnRAR source distributed through `unrar_sys` 0.5.8 solely to list
and extract explicitly declared RAR self-extracting archives. It never creates RAR archives.

UnRAR source code may be used in any software to handle RAR archives without limitations
free of charge, but cannot be used to develop RAR (WinRAR) compatible archiver and to
re-create RAR compression algorithm, which is proprietary. Distribution of modified UnRAR
source code in separate form or as a part of other software is permitted, provided that full
text of this paragraph, starting from "UnRAR source code" words, is included in license, or
in documentation if license is not available, and in source code comments of resulting
package.

Copyright Alexander L. Roshal. The UnRAR utility and source are distributed as-is, without
warranty. The Rust `unrar_sys` binding is available under MIT OR Apache-2.0.
