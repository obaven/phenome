use super::super::layout::GraphLayout;
use super::super::types::{
    GraphRenderRequest, GraphRenderStatus, TerminalImageProtocol,
};

#[derive(Debug)]
pub struct GraphRenderState {
    pub(crate) protocol: TerminalImageProtocol,
    pub(crate) request: Option<GraphRenderRequest>,
    pub(crate) cache_hash: Option<u64>,
    pub(crate) image: Option<Vec<u8>>,
    pub(crate) status: GraphRenderStatus,
    pub(crate) error: Option<String>,
    pub(crate) failed_hash: Option<u64>,
    pub(crate) image_id: u32,
    pub(crate) image_active: bool,
    pub(crate) layout: Option<GraphLayout>,
    pub(crate) layout_hash: Option<u64>,
    pub(crate) layout_error: Option<String>,
    pub(crate) selected_id: Option<String>,
    pub(crate) zoom: f64,
    pub(crate) pan_x: f64,
    pub(crate) pan_y: f64,
}

impl GraphRenderState {
    pub fn new() -> Self {
        Self {
            protocol: TerminalImageProtocol::detect(),
            request: None,
            cache_hash: None,
            image: None,
            status: GraphRenderStatus::Idle,
            error: None,
            failed_hash: None,
            image_id: 1,
            image_active: false,
            layout: None,
            layout_hash: None,
            layout_error: None,
            selected_id: None,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
        }
    }

    pub fn protocol(&self) -> TerminalImageProtocol {
        self.protocol
    }

    pub fn supports_images(&self) -> bool {
        matches!(
            self.protocol,
            TerminalImageProtocol::Kitty | TerminalImageProtocol::ITerm2
        )
    }

    pub fn protocol_label(&self) -> &'static str {
        self.protocol.label()
    }

    pub fn status(&self) -> GraphRenderStatus {
        self.status
    }

    pub fn last_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn request(&self) -> Option<&GraphRenderRequest> {
        self.request.as_ref()
    }

    pub fn image(&self) -> Option<&[u8]> {
        self.image.as_deref()
    }

    pub fn image_id(&self) -> u32 {
        self.image_id
    }

    pub fn image_active(&self) -> bool {
        self.image_active
    }

    pub fn set_image_active(&mut self, active: bool) {
        self.image_active = active;
    }

    pub fn clear_request(&mut self) {
        self.request = None;
    }
}
