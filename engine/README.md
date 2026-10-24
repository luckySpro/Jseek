# JSeek engine (C++)

Filename index, Everything-style query parser, WAL/persistence and content trigram index.
Imported from [JMacEverything](https://github.com/luckySpro/JMacEverything), itself a fork of
[joshua-wu/MacEverything](https://github.com/joshua-wu/MacEverything) (MIT). See the repo LICENSE.

M1 goal (docs/PLAN.md): remove macOS-only dependencies (GCD, CoreFoundation, F_NOCACHE…) from the
index/query/persistence core so the same tests pass on macOS, Windows and Linux. Scanning and
watching (getattrlistbulk, FSEvents) stay macOS-specific and move behind `jseek-platform`.
