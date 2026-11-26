/*
** EPITECH PROJECT, 2026
** EpiQuest
** File description:
** lib.rs
*/

/*
 ** EPITECH PROJECT, 2026
 ** lib.rs
 ** File description:
 ** SRD importer: Open5e -> versioned snapshot -> PostgreSQL
 */

pub mod error;
pub mod importer;
pub mod model;
pub mod normalize;
pub mod open5e;
pub mod snapshot;

pub use error::{Result, SrdError};
pub use importer::{ImportOptions, ImportOutcome, KindReport, import};
pub use model::{FORMAT_VERSION, Manifest, Snapshot};
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../api/migrations");
