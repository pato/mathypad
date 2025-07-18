# Changelog

All notable changes to this project will be documented in this file.

## [0.1.17] - 2025-07-14

### 🤖 AI Assisted
- Try fixing release script again
- Re-review release script


## [0.1.16] - 2025-07-14

### 🤖 AI Assisted
- Fix broken test
- Fix release script


### 👤 Artisanally Crafted



### Changes
- Release scripts


## [0.1.15] - 2025-07-12

### 🤖 AI Assisted
- Fix color scheme


### 👤 Artisanally Crafted

- Fix cargo publish



## [0.1.14] - 2025-07-09

### 🤖 AI Assisted
- Web POC
- Add script to start server
- Continue on the web POC
- Continue integrating mathypad-core
- Remove the syntax highlighted pane
- Online editor getting better
- Fix not being able to enter new line at end
- Fix new line on web
- Add syntax highlighting
- Fix line references
- Fix wasm container for website
- Fix refactor
- Improve Makefile
- Make wasm smaller
- Fix flickering
- Fix color scheme
- Fix safe area in PWA mode
- Icon
- Add htaccess to serve wasm properly
- Adding icons for PWA
- Added service worker
- Fix zoom
- Split up builds
- Add support for thousands with k (e.g., $100k)
- Add support for sum_above()


### 👤 Artisanally Crafted
- Manually bump dependency versions
- Gitignore
- Remove some text in mobile version



### Changes
- Cleanup


## [0.1.13] - 2025-06-25

### 🤖 AI Assisted
- Fix utf-8 character bug (deleting with utf8 characters was a mess)
- Add panic handler in case mathypad crashes (will save error logs and
- Add support for "quarter" as a time unit (e.g., 1 year - 1 quarter =
- Fixed syntax highltighting on line1/month


### 👤 Artisanally Crafted



## [0.1.12] - 2025-06-24

### 🤖 AI Assisted
- Stricter clippy
- Refactor: change DataRate variant to use named field (#3)
- Add support for exponents (i.e., 2^10)
- Add sqrt() support (with syntax highlighting)
- Add support for vim command mode (e.g., :w, :wq, :cq)
- Clean up code and add more vim motions (0, $, G, gg)
- Add support for currencies (e.g., $42 * 3)
- Add support for currency rates (e.g., $5/day * 3 months)
- Add support for currency / data rates (e.g., $5/GiB * 12 TiB)
- Fix currrency rate conversions (e.g., $500 / year to $/month)


### 👤 Artisanally Crafted
- Update CLAUDE.md and todo
- Update todos



## [0.1.11] - 2025-06-20

### 🤖 AI Assisted
- Check for clean working directory in release script
- Add a welcome message with the latest changes
- Add UI snapshot tests for welcome screen
- Add more vim key support (e.g., dd, x)
- Update website
- Add github release actions


### 👤 Artisanally Crafted
- Minor website tweaks
- Update readme ai section



## [0.1.10] - 2025-06-20

### 🤖 AI Assisted
- Add ui snapshot tests
- Add support for weeks, months, and years
- Add docs for the UI snapshot tests
- Improve the changelog generation
- Added ability to view changelog and save version
- Integrate changelog embedding into release process


### 👤 Artisanally Crafted
- Todo
- Changelog minor updates



### Changes
- Generic Rate Unit Support (#1)
-   Add rate unit addition and subtraction support (#2)


## [0.1.9] - 2025-06-19

### 🤖 AI Assisted
- Fix release script
- Add support for generating auto completion
- Improve its own code
- Fix clippy
- Allow dragging the border between typing and results
- Add support for copying expressions or results with double click


### 👤 Artisanally Crafted
- Update todo
- Update dialog to show Ctrl-C
- Claude.md
- Update todo



## [0.1.8] - 2025-06-12

### 🤖 AI Assisted
- Write a release script
- Add git push and cargo release to release script
- First pass at saving
- Confirmation dialog before quitting if not saved
- Pre-append the .pad extension when in save dialog


### 👤 Artisanally Crafted
- Todo



## [0.1.7] - 2025-06-12

### 🤖 AI Assisted
- Update readme
- Update the website
- Add github corner
- Fix variables not re-evaluating dependent lines
- Add support for percents
- Color the % and of
- Fix cursor in vim mode
- Add support for auto adapting line numbers
- Fix edge case with updating line references
- Still trying to fix edge cases in line reference updates
- Fix the automatic line reference updates
- Add project structure to CLAUDE.md
- Add support for loading files
- Fix empty line when loading files
- Add animations
- Make animations feel more smooth
- Add "-?" for help


### 👤 Artisanally Crafted
- Cargo fmt
- Try writing a CLAUDE.md
- Use .pad for file help



## [0.1.6] - 2025-06-11

### 🤖 AI Assisted
- Add support for expressions with units resulting in ratios
- Add support for variables
- Change tokenizer to parse text


### 👤 Artisanally Crafted
- Print the correct version with --version
- Optimize for small binary size



## [0.1.5] - 2025-06-10

### 🤖 AI Assisted
- Write chumksy parser
- Switch to using chumsky parser (with fallback)
- Fully replace with chumsky
- Fix parsing 5GiB (no space)
- Fix finding mathematical expressions
- Fix whitespace padded compound units


### 👤 Artisanally Crafted
- Add homepage to cargo manifest
- Add MIT license
- Fix homepage in Cargo.toml



## [0.1.4] - 2025-06-09

### 🤖 AI Assisted
- Add support for the Ctrl-W keybinding in insert mode
- Add support for subsecond units


### 👤 Artisanally Crafted
- Chaneglog



## [0.1.3] - 2025-06-08

### 🤖 AI Assisted
- Make a one page website!
- Add meta tags for better sharing
- Fix deleting empty lines


### 👤 Artisanally Crafted
- Move binary
- Ignore deploy script for website

- Cargo fmt


## [0.1.2] - 2025-06-08

### 🤖 AI Assisted
- Refactor like a senior engineer
- Fix clippy issues
- Restore the tests
- Add support for Gibps and GiBps
- Fix bug with MB vs Mib


### 👤 Artisanally Crafted
- Minor readme change
- Cargo fmt
- "s/-- NORMAL --/ NORMAL "



## [0.1.1] - 2025-06-08

### 🤖 AI Assisted
- Improve syntax highlighting
- Add vim motion support
- Move normal indicator to bottom


### 👤 Artisanally Crafted
- Change screenshot
- Fix typo in screenshot



## [0.1.0] - 2025-06-08

### 🤖 AI Assisted
- Let's handle commas
- Make "in" same as "to"
- Add ability to evaluate expressions from command line without tty
- Add support for referencing other lines
- Fix clippy warnings
- Review and cleanup
- Add support for QPS and req/s
- Add PiB and EiB
- Write claude.md


### 👤 Artisanally Crafted
- Cargo fmt
- Add todo of things to improve
- Cargo.toml
- Readme changes
- Cargo update + cargo upgrade


### Changes
- Let's start the project
- Add ctrl+c binding
- Draw digits in blue
- Lets start handling expressions
- Support text as well as equations
- Start to handle units
- Add unit tests
- Add unit tests
- Add support for "in" unit conversions
- Write readme



