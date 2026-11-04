//! Independent EDA foundation for the standalone PCBcraft application.
//!
//! PCBcraft owns its electronics model and session. Reusable UI and SolidCore
//! libraries belong behind explicit adapters, independent of the SolidCraft app.
//! The draft supports flat/hierarchical sheets and a small original symbol catalog.

mod definitions;
mod demo;
pub mod document;
pub mod error;
pub mod netlist;
pub mod page;
pub mod persistence;
pub mod probe;
pub mod project;
pub mod session;

pub mod wiring;

pub mod selection;
pub mod text;
