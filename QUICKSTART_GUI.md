# Quick Start - Tauri GUI

## ✅ Phase 11 Setup Complete!

You now have a working Tauri + React + TypeScript GUI!

## Running the GUI (2 Terminals)

### Terminal 1: Frontend Dev Server
```bash
cd ui
npm run dev
```

Wait until you see:
```
VITE v7.x.x  ready in XXXms
➜  Local:   http://localhost:5173/
```

### Terminal 2: Tauri App
```bash
# In the project root
cargo tauri dev
```

This will:
1. Compile the Rust backend
2. Launch the GUI window
3. Enable hot-reload for both frontend and backend

## What You'll See

A Windows desktop application with:
- **Green themed UI** (Tailwind CSS)
- **Test Connection card**: Enter your name and click "Greet"
- **Today's Schedule card**: Load your existing schedule data
- **Phase 11 Status**: List of completed features

## Features Working

✅ **React + TypeScript + Vite**
- Fast hot module replacement
- TypeScript type safety
- Component-based architecture

✅ **Tailwind CSS (Green Theme)**
- Primary color: `#22c55e` (green-500)
- Responsive design
- Utility-first CSS

✅ **Tauri Backend Integration**
- IPC communication working
- Can call Rust functions from React
- Access to existing scheduler library

✅ **Two Working Commands**:
1. `greet(name)` - Test IPC communication
2. `get_today_schedule()` - Load real schedule data

## Testing the Integration

1. **Test Greet**:
   - Enter your name in the input
   - Click "Greet" button
   - Should see: "Hello, [name]! Welcome to Scheduler."

2. **Test Schedule Loading**:
   - First, add some tasks using CLI:
     ```bash
     sched add "Test Task" --start 09:00 --end 10:00
     ```
   - Click "Load Schedule" button in GUI
   - Should see: Date, task count, completion rate

## Next Steps

### Add More IPC Commands

Edit `src-tauri/src/main.rs`:
```rust
#[tauri::command]
fn add_task(title: String, start: String, end: String) -> Result<(), String> {
    // Implementation
    Ok(())
}

// Register in main():
.invoke_handler(tauri::generate_handler![
    get_today_schedule,
    greet,
    add_task,  // New command
])
```

### Add More Components

Create `ui/src/components/Timeline.tsx`:
```typescript
export function Timeline() {
  // Visual timeline component
  return <div>Timeline View</div>
}
```

### Build for Production

```bash
# Build frontend
cd ui
npm run build

# Build Tauri app (creates installer)
cd ..
cargo tauri build
```

Output: `src-tauri/target/release/bundle/msi/Scheduler_2.0.0_x64_en-US.msi`

## Troubleshooting

### "Cannot find module '@tauri-apps/api/tauri'"
```bash
cd ui
npm install @tauri-apps/api@^1.5
```

### "beforeDevCommand terminated"
Run frontend first in Terminal 1, then Tauri in Terminal 2.

### Port 5173 already in use
```bash
# Kill the process
netstat -ano | findstr :5173
taskkill /PID <PID> /F
```

## File Structure

```
scheduler/
├── ui/                          # React frontend
│   ├── src/
│   │   ├── App.tsx             # Main component ✨
│   │   └── index.css           # Tailwind styles
│   ├── package.json
│   └── tailwind.config.js      # Green theme config
├── src-tauri/                   # Tauri backend
│   ├── src/
│   │   └── main.rs             # IPC commands ✨
│   ├── Cargo.toml
│   └── tauri.conf.json         # App config
└── src/                         # Existing Rust library
    └── ...                      # (used by both CLI and GUI)
```

## Development Tips

1. **Hot Reload**: Edit `App.tsx` and see changes instantly
2. **Console**: Open DevTools (F12) to see logs
3. **Rust Changes**: Tauri will auto-recompile on Rust file changes
4. **IPC Debugging**: Check console for IPC errors

## Current Status

- ✅ Basic GUI working
- ✅ IPC communication working
- ✅ Can load schedule data
- 🚧 Timeline view (not yet)
- 🚧 Dashboard charts (not yet)
- 🚧 System tray (not yet)
- 🚧 Notifications (not yet)

---

**🎉 Congratulations! You're now running Phase 11 Tauri GUI!**
