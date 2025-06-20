# Mathypad

A smart TUI calculator that understands units, variables, and makes complex calculations
simple within natural language text.

![Mathypad](./screenshots/screen1.png "Mathypad")

## What is it?

Mathypad is like a notepad where you can write math expressions with real-world
units and variables, and it automatically calculates results for you. Think "Google
calculator" or [Soulver][soulver] but for your terminal, with support for data
sizes, time, API performance metrics, and variable assignment.

[soulver]: https://soulver.app/

## Quick Start

```bash
# Cargo install
cargo install mathypad

# Clone and build
git clone https://github.com/pato/mathypad.git
cd mathypad
cargo build --release
cargo install --path .
```


```bash
# start the editor
mathypad
```

```bash
# Or use it directly from command line
mathypad -- "100 QPS * 1 hour"           # → 360,000 query
mathypad -- "5 GB to GiB"                # → 4.657 GiB
mathypad -- "Cost: 100 * 12 dollars"     # → 1,200
mathypad -- "1 TiB/s * 30 min"           # → 1,800 TiB
```

## Why You'll Love It

### Smart Unit Handling
No more mental math converting between GB and GiB, or calculating how many requests your API serves per day:

```
50 GiB/s * 1 hour                    → 180,000 GiB
100 QPS to requests per day          → 8,640,000 req
2 TB + 500 GB                        → 2,500 GB
1 PB to TiB                          → 909,494.702 TiB
```

### Real-World Problem Solving
Perfect for DevOps, data engineering, and capacity planning:

```bash
# How much storage for 6 months of logs?
100 MB/day * 180 days                → 18,000 MB

# API capacity planning
500 QPS * 1 year                     → 15,778,800,000 query

# Data transfer estimates  
10 Gbps to TB/hour                   → 4.5 TB/h

# Storage consolidation
5 TB + 2.5 TB + 1024 GiB             → 8.524 TB
```

### Variables and Multiline Calculations
Define variables and reference them in later calculations:

```
servers = 40                            → 40
ram = 1 TiB                            → 1 TiB  
servers * ram                          → 40 TiB
memory = 40 GiB                        → 40 GiB
time = 18 s                            → 18 s
memory / time                          → 2.222 GiB/s
```

### Natural Language Processing
Write mathematical expressions within natural text - punctuation, colons, and extra words are gracefully handled:

```
Cost: 100 * 12 dollars                → 1,200
Download: 1,000 MB at 50 MB/s          → 1,000 MB
Data center: 50 PB + 10 EB             → 10,050 PB
API performance: (100 QPS + 50 req/s) * 1 hour → 540,000 req
Transfer: 5 PB in 2 days               → 28.935 TB/s
```

### Interactive or One-Shot
Use it like a notepad with live results, or fire quick calculations from your terminal:

**Interactive Mode (with vim motions and variables):**
```
$ mathypad
┌─ Mathypad ────────────────────┬─ Results ─────────────┐
│   1 │ servers = 40            │   1 │ 40              │
│   2 │ ram = 2 GiB             │   2 │ 2 GiB           │
│   3 │ servers * ram           │   3 │ 80 GiB          │
│   4 │ Cost: line3 * $50       │   4 │ 4,000           │
└── NORMAL ─────────────────────┴─────┴─────────────────┘

• ESC → Normal mode (hjkl navigation, indicator at bottom)
• i/a/o → Insert mode for editing
• Normal mode: hjkl (movement), w/b (word movement), W/B (WORD movement), x (delete char), dd (delete line), dw/db/dW/dB (delete word)
• Ctrl+C/Ctrl+Q → Quit
```

**Command Line:**
```bash
mathypad -- "Cost: 1.5 EB / 100 Gbps"  → 33.333 h
```

## What It Handles

### Data Units
- **Decimal**: B, KB, MB, GB, TB, PB, EB
- **Binary**: KiB, MiB, GiB, TiB, PiB, EiB
- **Rates**: All the above + /s (e.g., GB/s, TiB/s)

### Performance Metrics
- **QPS**: queries per second, minute, hour
- **Request rates**: req/s, req/min, requests/hour
- **Load calculations**: QPS × time = total requests

### Time
- Seconds, minutes, hours, days
- Mix and match: `90 minutes + 1.5 hours = 240 min`

### Variables
- **Assignment**: `servers = 40`, `ram = 1 TiB`
- **References**: Use variables in calculations: `servers * ram`
- **Complex expressions**: `total = servers * ram + overhead`

## Complex Operations Made Simple

### Data Center Planning
```
# Total monthly bandwidth
1 PB/day * 30 days                   → 30 PB

# Storage requirement with replication  
100 TB * 3 replicas + 20% overhead   → 360 TB

# Network utilization
50 Gbps * 0.8 utilization to TB/day  → 432 TB/day
```

### API Performance Analysis
```
# Using variables for capacity planning
baseline = 250 QPS                   → 250 QPS
peak_multiplier = 2.5                → 2.5
peak_load = baseline * peak_multiplier → 625 QPS

# Monthly request volume
250 QPS * 30 days                    → 648,000,000 query

# Load balancer distribution
servers = 10                         → 10
total_qps = 5000 QPS                 → 5,000 QPS
per_server = total_qps / servers     → 500 QPS
```

### Mixed Unit Calculations
```
# Different storage types
2 TiB SSD + 8 TB HDD                 → 10.196 TB

# Cross-base conversions
1000 GB * 0.931 to GiB               → 931 GiB

# Rate × time calculations
100 MiB/s * 2 hours to GiB           → 703.125 GiB
```

### Conversion Chains
```
# Multi-step conversions
24 MiB * 32 servers in GB            → 0.805 GB
5000 queries / 10 minutes to QPS     → 8.333 QPS
(1 TiB + 500 GiB) / 8 hours          → 0.052 TiB/s
```

## Advanced Features

### Variables & Line References
Define and reference variables, plus reference previous line calculations:
```
# Variable definitions
servers = 40                         → 40
ram_per_server = 2 GiB               → 2 GiB
total_ram = servers * ram_per_server → 80 GiB

# Line references
Line 1: 100 TB
Line 2: line1 * 0.8                 → 80 TB
Line 3: line2 to TiB                → 72.760 TiB

# Combine variables and line references
storage_overhead = line3 * 0.2       → 14.552 TiB
total_storage = line3 + storage_overhead → 87.312 TiB
```

### Smart Parsing & Natural Language
It figures out what you mean from natural text with punctuation and extra words:
```
Transfer: 5 PB in 2 days             → 28.935 TB/s
API load during peak: 2000 QPS       → 2,000 QPS
Storage needed: 500 GB * 12 months    → 6,000 GB
Calculate: (5 GiB + 3 GiB) * 2       → 16 GiB
Download: 1,000 MB at 50 MB/s        → 1,000 MB
Bandwidth usage: 100 Mbps * 1 hour   → 360,000 Mb
```

### Flexible Syntax
Use "to" or "in" for conversions:
```
1 GB to MiB                          → 953.674 MiB
100 TB * 3 in PB                     → 0.3 PB
```

## Installation

Requires [Rust](https://rustup.rs/):

The simplest way is through cargo install

```bash
cargo install mathypad
```

Or build it from the repository

```bash
git clone https://github.com/pato/mathypad.git
cd mathypad
cargo build --release
```

### Shell Completions

Mathypad provides shell completions that enable tab-completion for `.pad` files.

Enable completions using the built-in `--completions` flag:

**Bash:**
```bash
# Add to ~/.bashrc
echo 'eval "$(mathypad --completions bash)"' >> ~/.bashrc
source ~/.bashrc
```

**Zsh:**
```bash
# Add to ~/.zshrc
echo 'eval "$(mathypad --completions zsh)"' >> ~/.zshrc
source ~/.zshrc
```

**Fish:**
```bash
# Add to ~/.config/fish/config.fish
echo 'mathypad --completions fish | source' >> ~/.config/fish/config.fish
```

With completions installed, typing `mathypad ` and pressing Tab will auto-complete `.pad` files in the current directory.

## Use Cases

- **DevOps**: Storage capacity planning, bandwidth calculations, infrastructure cost modeling
- **Data Engineering**: Dataset size estimates, transfer time calculations, ETL planning
- **API Design**: Rate limiting, capacity planning, load testing, scaling calculations
- **System Administration**: Resource allocation, performance monitoring, server planning
- **Financial Planning**: Cost calculations with variables for different scenarios
- **Research & Analysis**: Reusable calculations with variables and documentation
- **General Computing**: Any calculation involving data sizes, rates, or reusable parameters

## Why Not Just Use a Regular Calculator?

**Traditional approach - Unit conversions:**
```bash
# 1. Convert 1 PB to bytes: 1,000,000,000,000,000
# 2. Divide by bytes per GiB: 1,000,000,000,000,000 ÷ 1,073,741,824
# 3. Get: 931,322.574615479
```

**Traditional approach - Reusable calculations:**
```bash
# 1. Calculate servers: 40
# 2. Remember that number
# 3. Calculate RAM: 2 GB each
# 4. Multiply: 40 * 2 = 80 GB
# 5. Lose track of what 40 and 2 represented
```

**Mathypad way:**
```bash
# Unit conversions
1 PB to GiB                          → 931,322.575 GiB

# Reusable calculations with variables
servers = 40                         → 40
ram_per_server = 2 GiB               → 2 GiB  
total_ram = servers * ram_per_server → 80 GiB

# Natural language processing  
Cost: servers * $500 each            → 20,000
```

Much better, right?

## Development & Testing

### UI Snapshot Testing

Mathypad uses snapshot testing to ensure the terminal UI renders correctly across changes. This helps catch visual regressions and ensures consistent user experience.

**What are snapshot tests?**
Snapshot tests capture the exact visual output of the TUI and compare it against stored "golden" snapshots on future test runs. If the output changes, the test fails, allowing you to review whether the change was intentional.

**Running UI tests:**
```bash
# Run all UI snapshot tests
cargo test ui::

# Run a specific UI test
cargo test ui::tests::test_basic_ui_layout
```

**When tests fail due to intentional UI changes:**
```bash
# Install cargo-insta (if not already installed)
cargo install cargo-insta

# Review changes interactively - shows diffs and allows accept/reject
cargo insta review

# Or automatically accept all changes (use with caution)
cargo insta accept
```

**What gets tested:**
- Basic UI layout and panels
- Syntax highlighting for different elements (numbers, units, operators)
- Cursor positioning and highlighting
- Dialog rendering (save/unsaved dialogs)  
- Different app states (normal/insert mode, scrolling)
- Separator indicators and layout changes

The snapshot files are stored in `src/ui/snapshots/` and should be committed to version control. When making UI changes, always review the snapshot diffs to ensure they match your intentions.

## Developed using AI 🤖

This was developed using Claude Code (with 3.5 Haiku and Sonnet 4).

Every commit has alongside with it the prompt I used that generated the
contents of the commit (with the exception of commits marked as no ai, but
there was no code that wasn't written by the model).

For reference, the original V0 working version cost a grand total of $25.

```
> /cost
  ⎿  Total cost:            $25.60
     Total duration (API):  1h 45m 33.9s
     Total duration (wall): 23h 21m 24.6s
     Total code changes:    3518 lines added, 610 lines removed
     Token usage by model:
         claude-3-5-haiku:  405.9k input, 21.2k output, 0 cache read, 0 cache write
            claude-sonnet:  2.4k input, 213.9k output, 47.4m cache read, 2.1m cache write
```
