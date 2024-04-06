pub struct Swapchain {
    surface_config          : wgpu::SurfaceConfiguration,
    min_size                : (u32, u32),
    max_size                : (u32, u32),
}


impl Swapchain {

    pub async fn new(instance: &crate::GPUInstance) -> Result<Self, String> {

        let adapter = instance.adapter();
        let surface = instance.surface();

        let surface_caps = surface.get_capabilities(adapter);
        let (prefered_format, prefered_present_mode) = Self::get_prefered_format_and_present_mode(&surface_caps);

        let min_size = (800, 600);
        let max_size = (8000, 8000);

        let surface_config = wgpu::SurfaceConfiguration {
            usage           : wgpu::TextureUsages::RENDER_ATTACHMENT,
            format          : prefered_format,
            width           : 800,
            height          : 600,
            present_mode    : prefered_present_mode,
            alpha_mode      : surface_caps.alpha_modes[0],
            view_formats    : vec![],
        };

        surface.configure(instance.device(), &surface_config);


        Ok(Self {
            surface_config      : surface_config,
            min_size            : min_size,
            max_size            : max_size,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width.clamp(self.min_size.0, self.max_size.0);
        self.surface_config.height = height.clamp(self.min_size.1, self.max_size.1);
    }

    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_config
    }

    pub fn acquire_texture(&self, instance: &crate::GPUInstance) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        Ok(instance.surface().get_current_texture()?)
    }

    pub fn present(&self, surface_texture: wgpu::SurfaceTexture) -> Result<(), String> {
        surface_texture.present();
        Ok(())
    }


    fn get_prefered_format_and_present_mode(surface_caps: &wgpu::SurfaceCapabilities) -> (wgpu::TextureFormat, wgpu::PresentMode) {
        let formats = surface_caps.formats.clone();
        let present_modes = surface_caps.present_modes.clone();

        let format = formats.iter().find(|&format| {
            format.is_srgb()
        }).map(|&format| format).unwrap_or(formats[0]);

        let present_mode = present_modes.iter().find(|&mode| {
            *mode == wgpu::PresentMode::Mailbox
        }).map(|&mode| mode).unwrap_or(present_modes[0]);

        (format, present_mode)
    }

}