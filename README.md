<img src=".github/splash.png" alt="ESO Log Tool Splash Banner" />

[![Release](https://github.com/sheumais/logs/actions/workflows/release.yml/badge.svg?branch=release)](https://github.com/sheumais/logs/actions/workflows/release.yml)
[![Chat](https://img.shields.io/badge/chat-discord-5865f2.svg?logo=discord&logoColor=fff)](https://discord.gg/FjJjXHjUQ4)

This application is designed to help you manage and process your Elder Scrolls Online Encounter log files. 

### Core Features
- Log Fixes – automatically applies fixes to encounter logs
- ESO Logs Integration – upload logs directly to esologs.com
- Live Logging – upload logs in real time with fixes
- Split & Merge – combine and split logs up to 10x faster
- Lightweight & Fast - lower ram usage, faster processing

### Current Fixes  
- Scribing skills display their scripts
- Fatecarver casts now display in the cast list
- Show Touch of Z'en debuff stacks
- Show Bahsei's taint debuff stacks
- Better icon support, e.g. wall of elements
- **Fixed temp file writing crashes in live logging**
- **Auto-split live logs by zone changes**
- **Improved status message formatting with timestamps**

### Other useful log tools: 
[![Easy Stalking](https://img.shields.io/badge/AddOn-Easy%20Stalking-f03f36.svg?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIGZpbGw9Im5vbmUiIHZpZXdCb3g9IjAgMCA2MCA2MCI+PGcgY2xpcC1wYXRoPSJ1cmwoI2EpIj48cGF0aCBmaWxsPSJ1cmwoI2IpIiBkPSJNNDkgMS41SDExQTkuNSA5LjUgMCAwIDAgMS41IDExdjM4YTkuNSA5LjUgMCAwIDAgOS41IDkuNWgzOGE5LjUgOS41IDAgMCAwIDkuNS05LjVWMTFBOS41IDkuNSAwIDAgMCA0OSAxLjUiLz48cGF0aCBmaWxsPSJ1cmwoI2MpIiBkPSJNMzAgNDRjMTEuNiAwIDIxLTcuODQgMjEtMTcuNVM0MS42IDkgMzAgOSA5IDE2Ljg0IDkgMjYuNSAxOC40IDQ0IDMwIDQ0Ii8+PHBhdGggZmlsbD0iIzAwMCIgZD0iTTMwIDMxYTYgNiAwIDEgMCAwLTEyIDYgNiAwIDAgMCAwIDEyIi8+PHBhdGggZmlsbD0iI2ZmZiIgZD0iTTI4LjUgMjZhMi41IDIuNSAwIDEgMCAwLTUgMi41IDIuNSAwIDAgMCAwIDUiLz48bWFzayBpZD0iZSIgd2lkdGg9IjYwIiBoZWlnaHQ9IjYwIiB4PSIwIiB5PSIwIiBtYXNrVW5pdHM9InVzZXJTcGFjZU9uVXNlIiBzdHlsZT0ibWFzay10eXBlOmFscGhhIj48cGF0aCBmaWxsPSJ1cmwoI2QpIiBzdHJva2U9IiMwMDAiIHN0cm9rZS13aWR0aD0iMyIgZD0iTTQ5IDEuNUgxMUE5LjUgOS41IDAgMCAwIDEuNSAxMXYzOGE5LjUgOS41IDAgMCAwIDkuNSA5LjVoMzhhOS41IDkuNSAwIDAgMCA5LjUtOS41VjExQTkuNSA5LjUgMCAwIDAgNDkgMS41WiIvPjwvbWFzaz48ZyBmaWxsPSIjQkEyRDI0IiBtYXNrPSJ1cmwoI2UpIj48cGF0aCBkPSJNOCA0MGEzIDMgMCAxIDAgMC02IDMgMyAwIDAgMCAwIDZtLTQgNGEyIDIgMCAxIDAgMC00IDIgMiAwIDAgMCAwIDRtNTAgNWE1IDUgMCAxIDAgMC0xMCA1IDUgMCAwIDAgMCAxMCIvPjxwYXRoIGQ9Ik00OSA2MmE4IDggMCAxIDAgMC0xNiA4IDggMCAwIDAgMCAxNm0tMTQuNS01YTIuNSAyLjUgMCAxIDAgMC01IDIuNSAyLjUgMCAwIDAgMCA1Ii8+PC9nPjxwYXRoIHN0cm9rZT0iIzAwMCIgc3Ryb2tlLXdpZHRoPSIzIiBkPSJNNDkgMS41SDExQTkuNSA5LjUgMCAwIDAgMS41IDExdjM4YTkuNSA5LjUgMCAwIDAgOS41IDkuNWgzOGE5LjUgOS41IDAgMCAwIDkuNS05LjVWMTFBOS41IDkuNSAwIDAgMCA0OSAxLjVaIi8+PHBhdGggc3Ryb2tlPSIjZmZmIiBkPSJNNSAxMmMuMzMtMi4zMyAyLjItNyA3LTdtNDMgN2MtLjMzLTIuMzMtMi4yLTctNy03Ii8+PHBhdGggZmlsbD0iI2ZmZiIgZD0ibTE3IDQ1LTUgNCA2IDN6Ii8+PHBhdGggc3Ryb2tlPSIjNzAxNDA2IiBzdHJva2Utd2lkdGg9IjEuNSIgZD0iTTkgNDZjLjMzIDIuNSA1IDcuNSAyMSA3LjUiLz48L2c+PGRlZnM+PGxpbmVhckdyYWRpZW50IGlkPSJiIiB4MT0iMzAiIHgyPSIzMCIgeTE9IjAiIHkyPSI2MCIgZ3JhZGllbnRVbml0cz0idXNlclNwYWNlT25Vc2UiPjxzdG9wIHN0b3AtY29sb3I9IiNGNjkzOEQiLz48c3RvcCBvZmZzZXQ9Ii4yNCIgc3RvcC1jb2xvcj0iI0YwM0YzNiIvPjxzdG9wIG9mZnNldD0iLjg2IiBzdG9wLWNvbG9yPSIjRjAzRjM2Ii8+PHN0b3Agb2Zmc2V0PSIxIiBzdG9wLWNvbG9yPSIjODAyMzFFIi8+PC9saW5lYXJHcmFkaWVudD48bGluZWFyR3JhZGllbnQgaWQ9ImMiIHgxPSIzNyIgeDI9IjI0IiB5MT0iMTAiIHkyPSI0Mi41IiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSI+PHN0b3Agc3RvcC1jb2xvcj0iI2ZmZiIvPjxzdG9wIG9mZnNldD0iLjc3IiBzdG9wLWNvbG9yPSIjZmZmIi8+PHN0b3Agb2Zmc2V0PSIxIiBzdG9wLWNvbG9yPSIjRTdFOEU5Ii8+PC9saW5lYXJHcmFkaWVudD48bGluZWFyR3JhZGllbnQgaWQ9ImQiIHgxPSIzMCIgeDI9IjMwIiB5MT0iMCIgeTI9IjYwIiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSI+PHN0b3Agc3RvcC1jb2xvcj0iI0Y2OTM4RCIvPjxzdG9wIG9mZnNldD0iLjI0IiBzdG9wLWNvbG9yPSIjRjAzRjM2Ii8+PHN0b3Agb2Zmc2V0PSIuODYiIHN0b3AtY29sb3I9IiNGMDNGMzYiLz48c3RvcCBvZmZzZXQ9IjEiIHN0b3AtY29sb3I9IiM4MDIzMUUiLz48L2xpbmVhckdyYWRpZW50PjxjbGlwUGF0aCBpZD0iYSI+PHBhdGggZmlsbD0iI2ZmZiIgZD0iTTAgMGg2MHY2MEgweiIvPjwvY2xpcFBhdGg+PC9kZWZzPjwvc3ZnPg==)](https://www.esoui.com/downloads/info2332-EasyStalking-Encounterlog.html)

### Future Goals
- User interface improvements
- ESO Logs parser parity
- TFB Log Insights features replacement
- Additional speed & memory optimisation
- Multiple languages support

### Building & Contributing

Download and install [Rust](https://rustup.rs/)

Install the [Tauri CLI](https://v2.tauri.app/reference/cli/) using Cargo
```sh
cargo install tauri-cli --version "^2.0.0" --locked
```

Install [trunk](https://trunkrs.dev/)
```sh
cargo install --locked trunk
```

Add the WebAssembly target
```sh
rustup target add wasm32-unknown-unknown
```

Run the desktop application
```sh
cargo tauri dev
```

If you have any questions, concerns or suggestions feel free to join the discord.

## Disclaimer

By downloading or using this software, you acknowledge and agree to the following terms:

- Good Faith Effort: I have made a reasonable attempt to ensure that this application is safe, functional, and free of major bugs. However, due to the nature of software development, it is impossible to guarantee that the app will always work flawlessly in every environment or scenario.

- No Warranty: This software is provided "as-is," without any warranties or guarantees, express or implied. I do not guarantee that the application will be free from errors, bugs, or interruptions, nor that it will always meet your specific needs or expectations.

- Usage at Your Own Risk: While I have taken precautions to ensure the app works properly, you are solely responsible for any consequences that arise from using it. I am not liable for any direct, indirect, incidental, special, or consequential damages, including data loss or system issues, that may result from using or not being able to use the application.

- No Responsibility for Third-Party Interactions: The application may interact with third-party services, websites, or tools. I cannot guarantee the availability or safety of those external services, and I am not responsible for any issues that arise from their use.

- Compliance with Laws: It is your responsibility to ensure that using this software complies with any relevant laws, regulations, and policies in your region.
