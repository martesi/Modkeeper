### Codebase Overview
- **Language**: Rust (primary), TypeScript (frontend)
- **Build**: Cargo for Rust
- **Frontend**: Bun
- **Project**: Game mod management and development

### Development Guidelines
- **Testing**: Add tests for new features, refactor existing tests after changes
- **Code Duplication**: Check for existing logic before adding new code, prefer extraction
- **Tooling**: Use `cargo run --bin export_types` to update frontend bindings

### Frontend Specifics
- Bun is used as the package manager and dev runtime
- Use Bun commands (e.g., `bun run dev`) for frontend development
- `export_types` binary updates TypeScript bindings
