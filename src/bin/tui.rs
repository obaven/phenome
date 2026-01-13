use rotappo_adapter_bootstrappo::BootstrappoBackend;
use rotappo_application::Runtime;
use rotappo_domain::ActionRegistry;
use rotappo_ui_tui as tui;
use rotappo_ui_tui::app::AppContext;

fn main() -> anyhow::Result<()> {
    // 1. Initialize backend (Sync) - do this before starting any global runtime
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

    // Check config for single-binary mode (Config field missing, using env var fallback)
    if std::env::var("ROTAPPO_SINGLE_BINARY").is_ok() {
        // Spawn analytics-service using std::process::Command
        let mut cmd = std::process::Command::new("./target/debug/analytics-service");
        // Or find it in PATH or use absolute path
        // We assume it's built and in target/debug for dev

        // Set env vars/args if needed
        // Spawn detached? Or kill on exit?
        // Using `spawn()` creates a child. If TUI exits, child might remain unless we kill it.

        match cmd.spawn() {
            Ok(_child) => {
                // TUI lifecycle doesn't easily support cleanup hooks in main.
                // We rely on OS cleanup or manual kill.
            }
            Err(e) => {
                eprintln!("Failed to spawn analytics-service: {}", e);
            }
        }
    }

    // 2. Create runtime for TUI components that need it (like App::new's async connection)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // 3. Run TUI inside runtime
    rt.block_on(async { tui::start(runtime, context) })
}
