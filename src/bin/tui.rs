use rotappo_adapter_bootstrappo::BootstrappoBackend;
use rotappo_application::Runtime;
use rotappo_domain::ActionRegistry;
use rotappo_ui_tui as tui;
use rotappo_ui_tui::app::AppContext;

fn main() -> anyhow::Result<()> {
    let backend = BootstrappoBackend::from_env()?;
    let ports = backend.ports();
    let runtime = Runtime::new_with_ports(ActionRegistry::default(), ports.clone());
    let context = AppContext {
        host_domain: backend.config.network.host_domain.clone(),
        config_path: backend.config_path.clone(),
        assembly_path: backend.assembly_path.clone(),
        assembly_error: backend.assembly_error.clone(),
        live_status_error: backend
            .live_status
            .as_ref()
            .and_then(|live| live.last_error()),
        ports,
    };
    tui::start(runtime, context)
}
