# Instructions for Claude Code

* This is the backend for an e2ee (end-to-end encryption) chat service. The frontend uses vodozemac as a library for e2ee.
* SQL queries are written in diesel, and schemas for tables can be found in the @migrations folder
* The code base should pass all of the lints in `cargo clippy` which are set to be pedantic and to disallow `unwrap`