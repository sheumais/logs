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

## Summary of Changes Since Fork from minimal-status-messages Branch

This section documents the comprehensive enhancement work done to improve the ESO Log Tool's upload functionality and user experience.

### Core Architecture Changes

**1. Upload Processing Unification**
- **Before**: Regular upload processed entire log files as single reports, while live logging created separate reports per BEGIN_LOG
- **After**: Both modes now use consistent BEGIN_LOG separation approach
- **Impact**: Users get properly segmented reports that accurately reflect individual gaming sessions

**2. Zone Detection Enhancement** 
- **Before**: Report names used the last zone seen in the log section (often incorrect)
- **After**: Uses zone where first combat occurs via `determine_section_zone()` function
- **Impact**: Report descriptions accurately reflect where encounters took place

### User Experience Improvements

**3. Status Message System Overhaul**
- **Before**: Inconsistent timestamp formats, missing status updates
- **After**: All messages use standardized "YYYY-MM-DD HH:MM:SS:" prefix via `format_status_timestamp()`
- **Impact**: Clear, professional status reporting with consistent timing information

**4. Live Logging Wait Indicators**
- **Before**: No feedback when live logging was waiting for new data
- **After**: Shows "Waited X minutes for new log entries" at 2-minute intervals (2, 4, 6, 8...)
- **Impact**: Users know the system is active and how long it's been waiting

**5. File Path Visibility**
- **Before**: Users couldn't easily see which files were being processed
- **After**: Full file paths displayed when processing begins
- **Impact**: Clear confirmation of which log files are being uploaded

**6. UI Message Management**
- **Before**: Wait messages accumulated, cluttering the interface
- **After**: Wait messages replace previous ones instead of appending
- **Impact**: Clean, readable status console without message spam

### Technical Implementation Details

**New Functions Added:**
- `format_status_timestamp()`: Standardized timestamp formatting
- `determine_section_zone()`: Intelligent zone detection based on first combat
- `process_log_section()`: Complete upload pipeline for individual BEGIN_LOG sections

**Enhanced Functions:**
- `upload_log()`: Complete rewrite to process sections individually
- `live_log_upload()`: Added wait time tracking and better status messages
- UI message handling: Added overwrite logic for wait messages

**Data Flow Improvements:**
- Log files are now parsed line-by-line to detect BEGIN_LOG boundaries
- Each section creates its own temporary files and upload pipeline
- Zone names determined by analyzing combat events, not just zone changes
- Master tables and segments uploaded for each section to prevent blank reports

### Commit History Summary
1. **b2728a6**: Implement BEGIN_LOG separation for regular upload with timestamp prefixes
2. **5bbdf6d**: Fix zone name detection to use first combat location instead of last zone change  
3. **bcb2453**: Remove dash from report description format for cleaner spacing
4. **c2129e4**: Add wait time status messages for live logging
5. **b6b8ffd**: Fix wait message format to use past tense and consistent timestamp

### Validation Results
- **Functionality**: All features tested and working correctly
- **Performance**: No significant performance impact from enhanced processing
- **Reliability**: Proper error handling and cancellation support maintained
- **User Experience**: Significantly improved status reporting and feedback

### Testing Notes
- App successfully compiles and runs with `cargo tauri dev`
- All functionality implemented with proper error handling and status reporting
- Upload pipeline now consistent between live and non-live modes
- Zone detection correctly identifies combat locations
- Wait messages properly replace instead of accumulating