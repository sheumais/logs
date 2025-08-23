# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Prerequisites Setup
```sh
# Install Rust
# Download from: https://rustup.rs/

# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0" --locked

# Install trunk for web frontend
cargo install --locked trunk

# Add WebAssembly target
rustup target add wasm32-unknown-unknown
```

### Running the Application
```sh
# Run desktop application in development mode
cd desktop && cargo tauri dev
# Or from workspace root:
cargo tauri dev

# Build for production
cd desktop && cargo tauri build
```

### Testing and Validation
```sh
# Check compilation
cd desktop/src-tauri && cargo check
cd desktop && cargo check  # Frontend check

# Run tests
cd desktop/src-tauri && cargo test
```

## Architecture Overview

### Workspace Structure
This is a Rust workspace with 4 crates:
- **`desktop/`**: Yew-based web frontend (WebAssembly) 
- **`desktop/src-tauri/`**: Tauri backend (native Rust application)
- **`cli/`**: ESO log processing and esologs.com integration
- **`parser/`**: Core ESO encounter log parsing logic

### Application Architecture

**Desktop Application (Tauri + Yew)**
- Frontend: Yew web framework compiled to WebAssembly
- Backend: Tauri provides native desktop integration and API endpoints
- Communication: Frontend calls Tauri commands via `tauri-sys`

**Key Data Flow:**
1. User selects log files via Tauri file dialogs
2. Frontend routes to different screens (Homepage, Upload, Live Log, etc.)
3. Tauri backend processes logs using `cli` and `parser` crates
4. Results uploaded to esologs.com via HTTP API
5. Status updates sent to frontend via Tauri events

**Core Components:**
- **Log Processing**: `cli::log_edit` applies fixes, `cli::esologs_convert` processes for upload
- **Live Logging**: Monitors encounter log file for real-time processing
- **Upload System**: Creates separate esologs.com reports per BEGIN_LOG section
- **Authentication**: Cookie-based session management with esologs.com

### Key Tauri Commands
- `upload_log`: Process and upload log files (creates separate reports per BEGIN_LOG)
- `live_log_upload`: Real-time log monitoring and upload
- `login`/`logout`: esologs.com authentication
- File operations: `pick_and_load_file`, `modify_log_file`, etc.

### Status Message System
All status messages include timestamp prefixes in format: "YYYY-MM-DD HH:MM:SS: message"

Use `format_status_timestamp()` helper function for consistent formatting:
```rust
window.emit("upload_status", format!("{}Processing log file: {}", format_status_timestamp(), path)).ok();
```

### Upload Processing Logic
Both regular upload and live logging now use the same approach:
1. Parse log file and detect BEGIN_LOG boundaries  
2. Create separate esologs.com report for each BEGIN_LOG section
3. Use zone names and timestamps for descriptive report titles  
4. Process each section through `split_and_zip_log_by_fight`
5. Upload master tables and segments via `upload_master_table` and `upload_segment_and_get_next_id`

**Key Functions:**
- `upload_log()`: Main upload function that processes entire log files with BEGIN_LOG separation
- `process_log_section()`: Handles individual BEGIN_LOG sections with full upload pipeline
- `live_log_upload()`: Continuous monitoring version using same section-based approach

### ESO Log File Format
- CSV-like format with game events
- Key events: BEGIN_LOG (new session), BEGIN_COMBAT (encounter start), ZONE_CHANGED
- Timestamps are game time, converted to Unix timestamps for reports

### Important Implementation Notes
- Upload functions must process data through the full pipeline: log parsing → master table creation → segment upload
- Live logging differs from regular upload only in continuous file monitoring vs. single file processing  
- Both modes create separate reports per BEGIN_LOG to avoid large combined reports
- All status messages require timestamp prefixes for user visibility

### Recent Changes (2025-08-23)
✅ **Implemented BEGIN_LOG separation for regular upload**: `upload_log()` now processes log files section by section, creating separate reports per BEGIN_LOG event (matching live logging behavior)

✅ **Added timestamp prefixes to all status messages**: All status messages now use `format_status_timestamp()` to show "YYYY-MM-DD HH:MM:SS: message" format

✅ **Fixed blank reports issue**: Added complete `process_log_section()` function that includes full upload pipeline (master tables + segments) to prevent empty reports

✅ **Enhanced file path status messages**: Both upload and live logging modes now show full file paths when processing begins

### Testing Notes
- App successfully compiles and runs with `cargo tauri dev`
- All functionality implemented with proper error handling and status reporting
- Upload pipeline now consistent between live and non-live modes