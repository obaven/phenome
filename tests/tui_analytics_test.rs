// use rotappo_application::Runtime;
// use phenome_domain::ActionRegistry;
// use phenome_ports::PortSet;
// use rotappo_ui_tui::app::{App, AppContext};

// #[test]
// fn tui_initializes_analytics_state() {
//     let runtime = Runtime::new_with_ports(ActionRegistry::default(), PortSet::empty());
//     let context = AppContext::new("localhost", "config.yml", "assembly.yml", PortSet::empty());
//     let app = App::new(runtime, context);
//     assert!(app.analytics_metrics.is_none());
//     assert!(app.analytics_anomalies.is_none());
//     assert!(app.analytics_recommendations.is_none());
// }
