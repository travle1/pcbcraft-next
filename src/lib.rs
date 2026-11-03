//! Independent EDA foundation for the standalone PCBcraft application.
//!
//! PCBcraft owns its electronics model and session. Reusable UI and SolidCore
//! libraries belong behind explicit adapters, independent of the SolidCraft app.
//! The first draft supports one schematic sheet and a small original symbol catalog.

mod definitions;
mod demo;
pub mod document;
pub mod error;
pub mod netlist;
pub mod persistence;
pub mod session;

pub mod wiring;
