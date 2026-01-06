use crate::models::error::SError;
use camino::{Utf8Path, Utf8PathBuf};
use dunce::canonicalize;
use std::path::PathBuf;

macro_rules! define_paths {
    ($name:ident { $($field:ident : $default:expr),* $(,)? }) => {
        #[derive(Clone, Debug)]
        pub struct $name {
            $(pub $field: Utf8PathBuf,)*
        }

        impl $name {
            pub fn to_absolute(mut self, base: &Utf8Path) -> Self {
                $(self.$field = base.join(self.$field);)*
                self
            }

            pub fn new(base: &Utf8Path) -> Self {
                Self::default().to_absolute(base)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $($field: $default.into(),)*
                }
            }
        }
    };
}

define_paths!(ModPaths {
    folder: "manifest",
    file: "manifest/manifest.json",
});

define_paths!(SPTPathRules {
    client_plugins: "BepInEx/plugins",
    server_mods: "SPT/user/mods",
    server_dll: "SPT/SPT.Server.dll",
    server_exe: "SPT/SPT.Server.exe",
    client_exe: "EscapeFromTarkov.exe",
});

define_paths!(LibPathRules {
    backups: "backups",
    mods: "mods",
    staging: "staging",
    manifest: "manifest.toml",
    cache: "cache.toml",
});
#[derive(Clone, Debug)]
pub struct SPTPathCanonical {
    pub server_exe: PathBuf,
    pub client_exe: PathBuf,
}

impl SPTPathCanonical {
    pub fn from_spt_paths(paths: SPTPathRules) -> Result<Self, SError> {
        Ok(Self {
            server_exe: canonicalize(paths.server_exe)?,
            client_exe: canonicalize(paths.client_exe)?,
        })
    }
}
