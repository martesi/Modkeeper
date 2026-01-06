use std::path::Path;
use sysinfo::System;

pub struct ProcessChecker;

impl ProcessChecker {
    /// Performs the check. Takes a mutable ref to System to allow
    /// sysinfo to reuse internal buffers for performance.
    pub fn is_running<P: AsRef<Path>>(sys: &mut System, target_paths: &[P]) -> bool {
        // Refresh only what we need
        sys.refresh_processes();

        sys.processes().values().any(|p| {
            if let Some(exe_path) = p.exe() {
                // Check if the current process path matches any of our targets
                return target_paths.iter().any(|target| exe_path == target.as_ref());
            }
            false
        })
    }
}