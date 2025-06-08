# Mathypad - Mathematical Notepad with Unit Conversion

A mathematical notepad application built in Rust that provides real-time expression evaluation, comprehensive unit conversion, and syntax highlighting.

## Features

### Core Functionality
- **Interactive TUI Mode**: Text editor interface with syntax highlighting
- **One-shot CLI Mode**: Evaluate expressions directly from command line using `--`
- **Real-time Evaluation**: Mathematical expressions are evaluated and results shown in right panel
- **Line References**: Reference results from other lines (e.g., `line1 + 4 GiB`)

### Unit Support

#### Data Units
**Base 10 (Decimal):**
- Byte (B), Kilobyte (KB), Megabyte (MB), Gigabyte (GB)
- **Terabyte (TB), Petabyte (PB), Exabyte (EB)** ✨ *Recently Added*

**Base 2 (Binary):**
- Kibibyte (KiB), Mebibyte (MiB), Gibibyte (GiB), Tebibyte (TiB)
- **Pebibyte (PiB), Exbibyte (EiB)** ✨ *Recently Added*

#### Data Rate Units
**Base 10:** B/s, KB/s, MB/s, GB/s, TB/s, **PB/s, EB/s** ✨ *Recently Added*
**Base 2:** KiB/s, MiB/s, GiB/s, TiB/s, **PiB/s, EiB/s** ✨ *Recently Added*

#### Request Rate Units ✨ *Recently Added*
- **QPS (Queries Per Second)**: `qps`, `queries/s`, `queries/sec`
- **QPM (Queries Per Minute)**: `qpm`, `queries/min`, `queries/minute`
- **QPH (Queries Per Hour)**: `qph`, `queries/h`, `queries/hour`
- **Request Rates**: `req/s`, `req/min`, `req/h`, `rps`, `rpm`, `rph`
- **Request Counts**: `req`, `request`, `requests`, `query`, `queries`

#### Time Units
- Second (s), Minute (min), Hour (h), Day (day)

### Mathematical Operations

#### Basic Arithmetic
- Addition, subtraction, multiplication, division
- Parentheses for order of operations
- Support for decimal numbers and comma-separated thousands

#### Unit-Aware Arithmetic
- **Addition/Subtraction**: Compatible units (e.g., `1 GiB + 512 MiB`)
- **Multiplication**: 
  - Rate × Time = Data/Requests (e.g., `100 QPS * 1 hour = 360,000 query`)
  - Number × Unit = Unit
- **Division**: 
  - Data ÷ Time = Rate (e.g., `1000 GiB / 10 minutes = 1.667 GiB/s`)
  - Requests ÷ Time = Request Rate (e.g., `3600 queries / 1 hour = 1 QPS`)

#### Unit Conversions
- **"to" keyword**: `1 GiB to KiB`, `100 QPS to req/minute`
- **"in" keyword**: `24 MiB * 32 in KiB`, `5000 queries / 10 minutes in QPS`
- **Cross-base conversions**: Mix base-10 and base-2 units freely
- **Large scale conversions**: `1 EB to TiB`, `5 EiB to PB`

## Usage Examples

### Basic Math
```
2 + 3 * 4                    → 14
(100 + 50) / 2               → 75
1,234,567 / 1000             → 1,234.567
```

### Data Unit Conversions
```
1 TiB to GiB                 → 1,024 GiB
5 PB to TB                   → 5,000 TB
1.5 EB to PiB                → 1,332,268 PiB
2 EiB - 1024 PiB            → 1,024 PiB
```

### QPS and Request Rate Calculations
```
100 QPS * 1 hour            → 360,000 query
25 QPS to req/minute        → 1,500 req/min
5000 queries / 10 minutes   → 8.333 QPS
100 QPS + 50 req/s          → 200 req/s
```

### Data Transfer Calculations
```
1 PB / 1 hour               → 0.000 PB/s
10 GiB/s * 30 minutes       → 18,000 GiB
500 TB/s * 8 hours          → 14,400,000 TB
```

### Real-World Scenarios
```
API load: 1000 QPS * 5 minutes     → 300,000 query
Data center: 50 PB + 10 EB          → 10,050 PB
Backup rate: 100 TB/s * 8 hours     → 2,880,000 TB
Network: 10 PB/s to TB/s            → 10,000 TB/s
```

### Line References
```
Line 1: 10 GiB
Line 2: line1 + 4 GiB              → 14 GiB
Line 3: line2 * 2 to MiB           → 28,672 MiB
```

## Command Line Usage

### Interactive Mode
```bash
cargo run
```

### One-shot Mode
```bash
cargo run -- -- "100 QPS * 1 hour"                → 360,000 query
cargo run -- -- "5 PB to TB"                      → 5,000 TB
cargo run -- -- "1.5 EB to TiB"                   → 1,364,242.053 TiB
```

### Help
```bash
cargo run -- --help
cargo run -- --version
```

## Recent Additions ✨

### Large Data Units (PB, EB, PiB, EiB)
- Added support for Petabytes and Exabytes in both base-10 and base-2 formats
- Full arithmetic and conversion support for enterprise/data center scale calculations
- Rate units (PB/s, EB/s, PiB/s, EiB/s) for high-throughput scenarios

### QPS and Request Rate Units
- Comprehensive support for measuring API and service performance
- QPS (Queries Per Second), QPM (Queries Per Minute), QPH (Queries Per Hour)
- Request rate units (req/s, req/min, req/h) with various aliases
- Arithmetic operations: QPS × time = total requests, requests ÷ time = QPS
- Real-world scenarios: load balancing, capacity planning, performance monitoring

## Testing

The project includes comprehensive unit tests covering:
- Basic arithmetic operations
- Unit conversions and parsing
- Large data unit functionality (30 test functions, 100+ test cases)
- QPS and request rate calculations
- Edge cases and error handling
- Line reference functionality
- Real-world usage scenarios

Run tests with:
```bash
cargo test
```

## Architecture

- **TUI**: Built with `ratatui` and `crossterm`
- **CLI**: Command-line parsing with `clap`
- **Expression Parsing**: Shunting-yard algorithm with unit-aware tokenization
- **Unit System**: Hierarchical unit types with base value conversions
- **Color Highlighting**: Numbers (light blue), units (green), keywords "to"/"in" (yellow), operators (cyan), line references (magenta)

## Development Notes

- All unit conversions use precise base values to avoid floating-point errors
- The tokenizer handles special cases for "to"/"in" keywords vs unit names (e.g., "TB")
- Large numbers are formatted with comma separators for readability
- Cross-base conversions (base-10 ↔ base-2) are supported seamlessly
- Line references prevent circular dependencies and future line references