use crate::models::library::LibraryDTO;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Deserialize, Serialize, Type)]
pub struct LibrarySwitch {
    pub active: Option<LibraryDTO>,
    pub libraries: Vec<LibraryDTO>,
}
