//! Analytics panel renderers.

pub mod advisory;
pub mod timeline;

pub use advisory::insights::render_insights;
pub use advisory::recommendations::render_recommendations;
pub use timeline::historical::render_historical;
pub use timeline::predictions::render_predictions;
pub use timeline::realtime::render_realtime;
