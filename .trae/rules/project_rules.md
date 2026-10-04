# BiliDownloader Project Rules

## Verification Commands

After making code changes, run these commands to verify:

### Rust Backend
```powershell
cargo test   # 运行所有测试
cargo build  # 编译检查
```

### Frontend
```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force; npx vue-tsc --noEmit  # TypeScript 类型检查
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force; npx vite build         # 前端构建
```

### Full Tauri Build
```powershell
# 开发模式
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force; cargo tauri dev

# 生产构建
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass -Force; cargo tauri build
```

## Architecture

- Rust backend: `src-tauri/`
- Vue 3 frontend: `src/`
- Sidecar: yt-dlp (not bundled in dev, needs to be in PATH)