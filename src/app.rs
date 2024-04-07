pub struct App {
    config              : crate::ConfigStore,
    window              : crate::Window,
    gpu_instance        : crate::GPUInstance,
    renderer            : crate::Renderer,
    is_running          : bool,
    hotkey_manager      : Option<global_hotkey::GlobalHotKeyManager>,
}

impl App {

    pub async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self, String> {
        let config = crate::ConfigStore::new(
            dirs::home_dir().ok_or("Failed to get home dir!")?.join(".xettacast/app_config.yml").to_str().ok_or("Failed to join paths!")?.to_string(),
            Some(include_str!("./assets/config/default_app.yaml").to_string())).await?;

        let window = crate::Window::new(event_loop).await?;
        let instance = crate::GPUInstance::new(&window).await?;

        let renderer = crate::Renderer::new(&instance, instance.swapchain().ok_or("No Swapchain!")?.surface_config().format).await?;

        let mut obj = Self {
            config              : config,
            window              : window,
            gpu_instance        : instance,
            renderer            : renderer,
            is_running          : true,
            hotkey_manager      : None,
        };
        
        if let crate::AppConfigItem::Monitor(monitor) = obj.config.get("monitor")? {
            if let Some(monitor) = obj.window.find_monitor(&monitor) {
                obj.window.set_monitor(&monitor);
            }
            else {
                log::warn!("Monitor not found: {}", monitor);
                obj.window.set_monitor(&obj.window.get_primary_monitor());
            }
        }
        
        obj.setup_hotkey_manager()?;

        Ok(obj)
    }

    pub fn on_update(&mut self) -> Result<bool, String> {
        self.on_render()?;
        Ok(self.is_running)
    }

    pub fn on_render(&mut self) -> Result<(), String> {

        let mut encoder = self.gpu_instance.encoder("App::on_render");
        let instance = &self.gpu_instance;
        let swapchain = instance.swapchain().ok_or("No swapchain!")?;


        let surface_texture: wgpu::SurfaceTexture = match swapchain.acquire_texture(&self.gpu_instance) {
            Ok(surface_texture) => {surface_texture},
            Err(wgpu::SurfaceError::Lost) => {
                instance.configure_surface(swapchain.surface_config());
                return Ok(());
            },
            Err(e) => {
                log::error!("Failed to acquire texture: {}", e);
                return Ok(());
            }
        }.into();

        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear( wgpu::Color {r: 0.0, g: 0.0, b: 0.0, a: 0.0}),
                        store:  wgpu::StoreOp::Store,
                    }
                })
            ],
            depth_stencil_attachment: None,
            label: Some("render"),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        self.renderer.begin()?;

        // self.renderer.rect(0.0, 0.0, 1.0, 1.0);

        rpass = self.renderer.flush(rpass, &instance)?;
        drop(rpass);
        
        self.renderer.end(&instance)?;
        
        


        self.gpu_instance.submit(encoder);
        
        swapchain.present(surface_texture)?;



        Ok(())
    }
    
    pub fn global_update(&mut self) -> Result<(), String> {
        self.check_hotkey()?;

        Ok(())
    }

    pub fn on_event(&mut self, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                self.close();
            },
            winit::event::WindowEvent::Resized(size) => {
                self.gpu_instance.swapchain_mut().unwrap().resize(size.width, size.height);
                self.gpu_instance.reconfigure_surface();
                log::info!("Resized: {:?}", size);
            },
            _ => {}
        }
    
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

 
    pub fn close(&mut self) {
        self.is_running = false;
    }

    fn check_hotkey(&mut self) -> Result<(), String> {
        if let Ok(key) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
            if key.state == global_hotkey::HotKeyState::Pressed {
                // log::info!("{:?}", key);
                if self.window.is_visible() {
                    self.window.hide();
                }else {
                    self.window.show();
                }

            }
        }
        Ok(())
    }

    fn setup_hotkey_manager(&mut self) -> Result<(), String> {
        if self.hotkey_manager.is_none() {
            self.hotkey_manager = Some(global_hotkey::GlobalHotKeyManager::new().map_err(|e| format!("Failed to create hotkey manager: {}", e))?);
        }
        if let crate::AppConfigItem::Trigger(trigger) = self.config.get("trigger")? {
            self.hotkey_manager.as_mut().ok_or("Failed to access hotkey manager!")?.register(trigger).map_err(|e| format!("Failed to register hotkey: {}", e))?;
        }
        Ok(())
    }
}