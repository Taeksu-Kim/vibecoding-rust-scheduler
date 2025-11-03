# Tauri GUI Setup Guide

This guide explains how to set up and develop the Tauri GUI client for Scheduler (Phase 11).

## Prerequisites

### Required Tools

1. **Rust** (already installed)
   - Version: 1.75+
   - Edition: 2021

2. **Node.js** (v18+)
   ```powershell
   # Download from https://nodejs.org/
   # Or use Chocolatey:
   choco install nodejs
   ```

3. **Tauri CLI** (already installed)
   ```bash
   cargo install tauri-cli --version "^1.5"
   ```

4. **WebView2** (Windows)
   - Usually pre-installed on Windows 10/11
   - Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

### Optional Tools

- **WiX Toolset** (for MSI installer)
  ```powershell
  # Download from https://wixtoolset.org/
  ```

## Project Structure

```
scheduler/
├── src/                    # Rust backend (existing)
├── src-tauri/             # Tauri-specific Rust code
│   ├── src/
│   │   └── main.rs        # Tauri app entry point
│   ├── tauri.conf.json    # Tauri configuration
│   ├── Cargo.toml         # Tauri dependencies
│   └── icons/             # App icons
└── ui/                    # Frontend (to be created)
    ├── src/
    │   ├── App.tsx        # Main component
    │   ├── components/    # React components
    │   └── styles/        # CSS/Tailwind
    ├── package.json
    └── vite.config.ts
```

## Initialization Steps

### 1. Initialize Tauri Project

```bash
# From project root
cargo tauri init

# Answer prompts:
# - App name: Scheduler
# - Window title: Scheduler - Daily Task Manager
# - Web assets: ../ui/dist
# - Dev server URL: http://localhost:5173
# - Frontend dev command: npm run dev
# - Frontend build command: npm run build
```

### 2. Choose Frontend Framework

#### Option A: React + TypeScript (Recommended)

```bash
# Create Vite project
npm create vite@latest ui -- --template react-ts

cd ui
npm install

# Install dependencies
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p

# Install UI libraries
npm install @tanstack/react-query
npm install @tauri-apps/api
npm install lucide-react  # Icons
npm install recharts       # Charts
npm install date-fns       # Date utilities
```

#### Option B: Vue + TypeScript

```bash
npm create vite@latest ui -- --template vue-ts
cd ui
npm install
# ... similar setup
```

#### Option C: Svelte + TypeScript

```bash
npm create vite@latest ui -- --template svelte-ts
cd ui
npm install
# ... similar setup
```

### 3. Configure Tailwind CSS (Green Theme)

**tailwind.config.js**:
```javascript
/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#f0fdf4',
          100: '#dcfce7',
          200: '#bbf7d0',
          300: '#86efac',
          400: '#4ade80',
          500: '#22c55e',  // Main green
          600: '#16a34a',
          700: '#15803d',
          800: '#166534',
          900: '#14532d',
        },
      },
    },
  },
  plugins: [],
}
```

### 4. Update Tauri Configuration

**src-tauri/tauri.conf.json**:
```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:5173",
    "distDir": "../ui/dist"
  },
  "package": {
    "productName": "Scheduler",
    "version": "2.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "notification": {
        "all": true
      },
      "systemTray": {
        "all": true
      }
    },
    "bundle": {
      "active": true,
      "identifier": "com.scheduler.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ],
      "windows": {
        "certificateThumbprint": null,
        "digestAlgorithm": "sha256",
        "timestampUrl": ""
      }
    },
    "security": {
      "csp": null
    },
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "Scheduler",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600
      }
    ],
    "systemTray": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  }
}
```

### 5. Link Existing Rust Backend

**src-tauri/Cargo.toml**:
```toml
[dependencies]
tauri = { version = "1.5", features = [ "system-tray", "notification-all", "shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Re-export existing scheduler library
scheduler = { path = "../" }
```

**src-tauri/src/main.rs**:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{CustomMenuItem, SystemTray, SystemTrayMenu, SystemTrayEvent};
use tauri::Manager;

// Import existing scheduler library
use scheduler::{Config, JsonStorage, Storage, Schedule, Task};

// Tauri commands
#[tauri::command]
fn get_today_schedule() -> Result<Option<Schedule>, String> {
    let storage = JsonStorage::new().map_err(|e| e.to_string())?;
    storage.load_today().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_task(
    title: String,
    start: String,
    end: String,
    tags: Vec<String>,
    notes: Option<String>
) -> Result<(), String> {
    // Implementation here
    Ok(())
}

fn main() {
    // System tray
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_today_schedule,
            add_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Development Workflow

### Run Dev Server

```bash
# Terminal 1: Start frontend dev server
cd ui
npm run dev

# Terminal 2: Start Tauri app
cargo tauri dev
```

### Build for Production

```bash
# Build frontend
cd ui
npm run build

# Build Tauri app (creates MSI installer)
cd ..
cargo tauri build
```

Output will be in `src-tauri/target/release/bundle/`

## Core Components to Implement

### 1. Timeline Component

**ui/src/components/Timeline.tsx**:
```typescript
import { useQuery } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';

interface Task {
  id: string;
  title: string;
  start_time: string;
  end_time: string;
  status: 'Pending' | 'InProgress' | 'Completed';
}

export function Timeline() {
  const { data: schedule } = useQuery({
    queryKey: ['schedule'],
    queryFn: () => invoke('get_today_schedule'),
  });

  return (
    <div className="timeline">
      {/* Timeline visualization */}
    </div>
  );
}
```

### 2. Dashboard Component

**ui/src/components/Dashboard.tsx**:
```typescript
export function Dashboard() {
  return (
    <div className="grid grid-cols-3 gap-4 p-6">
      <StatCard title="Efficiency" value="85%" />
      <StatCard title="Earned Time" value="4.5h" />
      <StatCard title="Streak" value="7 days" />
    </div>
  );
}
```

### 3. Task Editor

**ui/src/components/TaskEditor.tsx**:
```typescript
import { invoke } from '@tauri-apps/api/tauri';

export function TaskEditor() {
  const handleSubmit = async (data) => {
    await invoke('add_task', {
      title: data.title,
      start: data.start,
      end: data.end,
      tags: data.tags,
      notes: data.notes,
    });
  };

  return (
    <form onSubmit={handleSubmit}>
      {/* Task form fields */}
    </form>
  );
}
```

## Windows-Specific Features

### Auto-start with Windows

Add to `tauri.conf.json`:
```json
{
  "tauri": {
    "bundle": {
      "windows": {
        "wix": {
          "template": "installer.wxs"
        }
      }
    }
  }
}
```

### Global Hotkeys

```typescript
import { register } from '@tauri-apps/api/globalShortcut';

await register('CommandOrControl+Shift+S', () => {
  // Show quick add dialog
});
```

### Native Notifications

```typescript
import { sendNotification } from '@tauri-apps/api/notification';

sendNotification({
  title: 'Task Starting',
  body: 'Deep Work Session starts in 5 minutes',
});
```

## Testing

```bash
# Rust backend tests
cargo test

# Frontend tests
cd ui
npm run test

# E2E tests (with Playwright)
npm run test:e2e
```

## Packaging

### Create MSI Installer

```bash
cargo tauri build --bundles msi
```

### Code Signing (Optional)

1. Get a code signing certificate
2. Update `tauri.conf.json`:
```json
{
  "tauri": {
    "bundle": {
      "windows": {
        "certificateThumbprint": "YOUR_THUMBPRINT"
      }
    }
  }
}
```

## Next Steps

1. ✅ Install prerequisites
2. ✅ Initialize Tauri project
3. ✅ Choose frontend framework (React recommended)
4. ✅ Setup Tailwind with green theme
5. ⏳ Implement Timeline component
6. ⏳ Implement Dashboard
7. ⏳ Add system tray
8. ⏳ Implement notifications
9. ⏳ Build and test MSI installer

## Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Tauri Examples](https://github.com/tauri-apps/tauri/tree/dev/examples)
- [React + Tauri Template](https://github.com/tauri-apps/tauri-vite-react-template)
- [Tailwind CSS](https://tailwindcss.com/)
- [Recharts](https://recharts.org/)

---

**Status**: Phase 11 setup documentation complete. Ready for GUI development.
