pub struct GPUInstance {
    instance        : wgpu::Instance,
    adapter         : wgpu::Adapter,
    device          : wgpu::Device,
    queue           : wgpu::Queue,
    surface         : wgpu::Surface,
}


impl GPUInstance {
    pub async fn new(window: &crate::Window) -> Result<Self, String> {
        let instance = wgpu::Instance::new( wgpu::InstanceDescriptor {
            backends        : wgpu::Backends::PRIMARY,
            flags           : wgpu::InstanceFlags::empty(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(window.winit_handle()) }.map_err(|e| format!("Failed to create surface: {}", e))?;

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference            : wgpu::PowerPreference::HighPerformance,
            compatible_surface          : Some(&surface),
            force_fallback_adapter      : false,
        }).await.ok_or("Failed to find an appropriate adapter")?;

        let mut limits = wgpu::Limits::default();
        let mut features = wgpu::Features::empty();


        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features   : features,
                limits     : limits,
                label      : Some("GPUInstance"),
            },
            None,
        ).await.map_err(|e| format!("Failed to create device: {}", e))?;

        Ok(Self {
            instance            : instance,
            adapter             : adapter,
            device              : device,
            queue               : queue,
            surface             : surface,
        })
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn configure_surface(&self, config: &wgpu::SurfaceConfiguration) {
        self.surface.configure(&self.device, config);
    }

    pub fn encoder(&self, label: &str) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some(label),
        })
    }

    pub fn submit(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);
    }

    pub fn submit_without_poll(&self, encoder: wgpu::CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn poll(&self) {
        self.device.poll(wgpu::Maintain::Wait);
    }

}