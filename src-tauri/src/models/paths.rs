use camino::{Utf8Path, Utf8PathBuf};

macro_rules! define_paths {
    ($name:ident { $($field:ident : $default:expr),* $(,)? }) => {
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

define_paths!(SPTPaths {
    client_plugins: "BepInEx/plugins",
    server_mods: "SPT/user/mods",
    server_dll: "SPT/SPT.Server.dll",
    server_exe: "SPT/SPT.Server.exe",
    client_exe: "EscapeFromTarkov.exe",
});

define_paths!(LibPaths {
    backups: "backups",
    mods: "mods",
    staging: "staging",
    manifest: "manifest.toml",
    cache: "cache.toml",
});