pub struct App {
    config              : crate::ConfigStore,
    window              : crate::Window,
    gpu_instance        : crate::GPUInstance,
    renderer            : crate::Renderer,
    is_running          : bool,
    hotkey_manager      : Option<global_hotkey::GlobalHotKeyManager>,
    texture_packer      : crate::TexturePacker,
}

impl App {

    pub async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self, String> {
        let config = crate::ConfigStore::new(
            dirs::home_dir().ok_or("Failed to get home dir!")?.join(".xettacast/app_config.yml").to_str().ok_or("Failed to join paths!")?.to_string(),
            Some(include_str!("./assets/config/default_app.yaml").to_string())).await?;

        let window = crate::Window::new(event_loop).await?;
        let instance = crate::GPUInstance::new(&window).await?;

        let renderer = crate::Renderer::new(&instance, instance.swapchain().ok_or("No Swapchain!")?.surface_config().format).await?;

        let mut packer = crate::TexturePacker::new(4096, 4096, 2).await?;

        

        //let bytes = include_bytes!("C:\\Windows\\Fonts\\ariblk.ttf");
        let bytes = include_bytes!("../MaterialDesignIconsDesktop.ttf");
        let font = crate::Font::new(bytes, 32.0f32).await.unwrap();


        font.save_bitmaps("p").unwrap();

        font.glyphs().iter().for_each(|(id, glyph)| {
            packer.add(&format!("glyph{}", id), glyph.width, glyph.height);
        });

        // packer.add("asds", 10, 5);
        
        log::info!("Packing : {:?}", packer.pack());
        packer.fill_color()?;
        packer.save(&format!("p/packed{}.png", 0), 0)?;
        packer.save(&format!("p/packed{}.png", 1), 1)?;

        packer.fill_color_empty()?;
        packer.save(&format!("p/empty{}.png", 0), 0)?;
        packer.save(&format!("p/empty{}.png", 1), 1)?;

        let mut obj = Self {
            config              : config,
            window              : window,
            gpu_instance        : instance,
            renderer            : renderer,
            is_running          : true,
            hotkey_manager      : None,
            texture_packer      : packer,
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

        self.renderer.begin()?;
        self.renderer.set_frame_res(self.window.get_size());
        self.renderer.set_target_view(view);



        // self.renderer.rect(0.1, 0.1, 0.7, 0.9);
        // for i in 0..10000 {
        //     let random_x = rand::random::<f32>();
        //     let random_y = rand::random::<f32>();

        //     self.renderer.set_color(random_x, random_y, 0.0, 1.0);
        //     self.renderer.rect(random_x, random_y, 0.1, 0.1);
            
        //     // self.renderer.set_color(1.0, 1.0, 0.0, 1.0);
        //     // self.renderer.rectp(100, 100, 200, 400);
        // }
/*
        // self.renderer.set_border_radius(0.1, 0.2, 0.3, 0.4);
        // self.renderer.set_maskp(50, 150, 200, 300);
        self.renderer.set_border_radius(0.2, 0.2, 0.4, 1.0);
        self.renderer.set_color(0.4, 0.0, 0.0, 1.0);
        self.renderer.rectp(95, 195, 210, 110);
        self.renderer.set_color(1.0, 1.0, 0.0, 1.0);
        self.renderer.rectp(100, 200, 200, 100);
        
        // self.renderer.set_border_radius(0.2, 0.2, 0.4, 1.0);
        //self.renderer.set_border_radius(0.4, 0.2, 0.4, 0.0);
        self.renderer.rectp(200, 400, 300, 200);
*/

        // // draw 32x32 bitmap using rects of 20x20
        // let mut x = 0;
        // let mut y = 0;
        // for pixel in self.font.iter() {
        //     let color = *pixel as f32 / 255.0;
        //     self.renderer.set_color(color, color, color, 1.0);
        //     self.renderer.rectp(100 + 10 * x, 100+ 10 * (31-y), 10, 10);

        //     x += 1;
        //     if x == 32 {
        //         x = 0;
        //         y += 1;
        //     }
        // }

        
        self.renderer.end(&instance)?;
        
        
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