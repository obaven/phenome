//! Core application state for the TUI.

use ratatui::widgets::ListState;
use std::path::PathBuf;
use std::time::Instant;
// use tokio::sync::mpsc;

use crate::app::{GraphRenderState, NavSection, NavView};
use crate::state::UiState;
use phenome_application::Runtime;
use phenome_domain::{ActionId, ActionSafety, Anomaly, MetricSample, Recommendation};
use phenome_ports::PortSet;

use crate::analytics_client::AnalyticsClient;
/// External context required to run the TUI.
#[derive(Clone)]
pub struct AppContext {
    pub host_domain: String,
    pub config_path: PathBuf,
    pub assembly_path: PathBuf,
    pub assembly_error: Option<String>,
    pub live_status_error: Option<String>,
    pub ports: PortSet,
}

impl AppContext {
    /// Build a minimal context without action/live errors.
    pub fn new(
        host_domain: impl Into<String>,
        config_path: impl Into<PathBuf>,
        assembly_path: impl Into<PathBuf>,
        ports: PortSet,
    ) -> Self {
        Self {
            host_domain: host_domain.into(),
            config_path: config_path.into(),
            assembly_path: assembly_path.into(),
            assembly_error: None,
            live_status_error: None,
            ports,
        }
    }
}

/// Main TUI application state.
///
/// # Examples
/// ```rust,no_run
/// use phenome_application::Runtime;
/// use phenome_domain::ActionRegistry;
/// use phenome_ui_tui::app::App;
/// use phenome_ui_tui::app::AppContext;
/// use phenome_ports::PortSet;
///
/// let runtime = Runtime::new_with_ports(ActionRegistry::default(), PortSet::empty());
/// let context = AppContext::new("localhost", "config.yml", "assembly.yml", PortSet::empty());
/// let app = App::new(runtime, context);
/// assert!(!app.should_quit);
/// ```
pub struct App {
    pub runtime: Runtime,
    pub context: AppContext,
    pub action_state: ListState,
    pub confirm: Option<ConfirmPrompt>,
    pub last_refresh: Instant,
    pub should_quit: bool,
    pub ui: UiState,
    pub graph: GraphRenderState,
    pub active_nav: NavSection,
    pub active_view: NavView,
    pub nav_sub_index: [usize; 3],
    pub analytics_metrics: Option<Vec<MetricSample>>,
    pub analytics_anomalies: Option<Vec<Anomaly>>,
    pub analytics_recommendations: Option<Vec<Recommendation>>,
    pub analytics_cache_timestamp: Option<Instant>,
    pub analytics_client: Option<AnalyticsClient>,
    pub analytics_rx: Option<tokio::sync::mpsc::Receiver<AnalyticsUpdate>>,
}

#[derive(Debug)]
pub enum AnalyticsUpdate {
    Metrics(Vec<MetricSample>),
    Anomalies(Vec<Anomaly>),
    Recommendations(Vec<Recommendation>),
}

/// Confirmation prompt details for high-risk actions.
#[derive(Debug, Clone)]
pub struct ConfirmPrompt {
    pub action_id: ActionId,
    pub label: String,
    pub safety: ActionSafety,
}