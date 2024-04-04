pub struct Window {
    winit_window: winit::window::Window,
}

impl Window {
    pub async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self, String> {
        let winit_window = winit::window::WindowBuilder::new()
            .with_title("Global Hotkey")
            .with_content_protected(true)
            .with_visible(false)
            .with_resizable(false)
            // .with_decorations(false)
            // .with_transparent(true)
            .build(event_loop)
            .map_err(|e| format!("Failed to create window: {}", e))?;
        
        Ok(Self {
            winit_window        : winit_window,
        })
    }

    pub fn id(&self) -> winit::window::WindowId {
        self.winit_window.id()
    }

    pub fn request_redraw(&self) {
        self.winit_window.request_redraw();
    }

    pub fn get_available_monitors(&self) -> Vec<winit::monitor::MonitorHandle> {
        self.winit_window.available_monitors().collect()
    }

    pub fn get_active_monitor(&self) -> winit::monitor::MonitorHandle {
        self.winit_window.current_monitor().unwrap()
    }

    pub fn get_primary_monitor(&self) -> winit::monitor::MonitorHandle {
        self.winit_window.primary_monitor().unwrap()
    }

    pub fn find_monitor(&self, name: &str) -> Option<winit::monitor::MonitorHandle> {
        if name == "primary" {
            Some(self.get_primary_monitor())
        } else {
            self.winit_window.available_monitors().find(|monitor| monitor.name().map(|n| n == name).unwrap_or(false))
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.winit_window.scale_factor()
    }

    pub fn get_position(&self) -> (i32, i32) {
        self.winit_window.outer_position().unwrap().to_logical::<i32>(self.scale_factor()).into()
    }

    pub fn set_position(&self, x: i32, y: i32) {
        self.winit_window.set_outer_position(winit::dpi::LogicalPosition::new(x, y));
    }

    pub fn get_size(&self) -> (u32, u32) {
        let size = self.winit_window.inner_size();
        (size.width, size.height)
    }

    pub fn set_size(&self, width: u32, height: u32) {
        self.winit_window.set_inner_size(winit::dpi::LogicalSize::new(width, height));
    }

    pub fn get_monitor_with_position(&self, x: i32, y: i32) -> Result<winit::monitor::MonitorHandle, ()> {
        self.winit_window
            .available_monitors()
            .find(|monitor| {
                let position = monitor.position();
                let size = monitor.size().to_logical::<i32>(self.scale_factor());
                x >= position.x && x < position.x + size.width && y >= position.y && y < position.y + size.height
            }).ok_or(())
    }

    pub fn set_monitor(&self, monitor: &winit::monitor::MonitorHandle) {
        // if self.is_minimized() {
        //     self.restore();
        // }

        // if !self.is_visible() {
        //     self.show();
        // }

        self.set_size(monitor.size().width, monitor.size().height);
        self.set_position(monitor.position().x, monitor.position().y);
        // self.winit_window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(Some(monitor.clone()))));
    }

    pub fn minimize(&self) {
        self.winit_window.set_minimized(true);
    }

    pub fn restore(&self) {
        self.winit_window.set_minimized(false);
    }

    pub fn is_minimized(&self) -> bool {
        self.winit_window.is_minimized().unwrap()
    }

    pub fn is_visible(&self) -> bool {
        self.winit_window.is_visible().unwrap()
    }

    pub fn hide(&self) {
        self.winit_window.set_visible(false);
    }

    pub fn show(&self) {
        self.winit_window.set_visible(true);
    }
    

}