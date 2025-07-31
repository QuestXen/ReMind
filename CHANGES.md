# ReMind Changelog

## Version 1.0.5
*Released on: July 31, 2025*

### New Features
**Input Handling**
- Added support for decimal numbers (e.g., 1.5) in interval value fields
- Enhanced interval input to accept fractional values for more precise timing

### Improvements
**Timer Management**
- Implemented intelligent timer reset logic that only restarts timers when interval or count values change
- Timer now preserves its state when editing non-timing related properties like colors or titles
- Optimized reminder update process to avoid unnecessary timer resets

### Bug Fixes
**UI/UX**
- Fixed flickering issue with "next reminder" display when a reminder is executed
- Ensured the next reminder time remains visible without disappearing momentarily
- Resolved UI container flickering where the next reminder display briefly disappears and reappears
- Improved visual stability of reminder display during updates

**Performance**
- Optimized reminder update process to avoid unnecessary reloading of all reminders
- Improved event handling for reminder execution to update only affected reminders

**Startup & Connectivity**
- Fixed "localhost server not found" error after PC restart when app auto-starts
- Added startup delays to ensure frontend server is ready before initialization
- Improved error handling and retry logic for connection issues during app startup
- Enhanced app initialization sequence to prevent connectivity problems

---

## Version 1.0.0
*Released on: July 30, 2025*

### New Features
**Core Application**
- Application successfully released and fully functional with all core features.




