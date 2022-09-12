# Playlist Fixer

This app detects invalid entries in an `.m3u8` file and fixes them. The entries are invalid due to changes in their extension names.

To fix the entries, the app replaces the extension names of the invalid entries with the new one specified by the user. If the fixed entry is still not valid, the user will be warned.

This Rust implementation is a rewrite and upgrade of https://github.com/czyczk/playlist_fixer_go.
