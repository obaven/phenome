//! Analytics panel renderers.

pub mod historical;
pub mod insights;
pub mod predictions;
pub mod realtime;
pub mod recommendations;

pub use historical::render_historical;
pub use insights::render_insights;
pub use predictions::render_predictions;
pub use realtime::render_realtime;
pub use recommendations::render_recommendations;
