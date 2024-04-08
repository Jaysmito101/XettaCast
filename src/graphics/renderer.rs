#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]  #[repr(C)]
struct RendererItem {
    offset_scale            : [f32; 4], // (x, y), (scale_x, scale_y)
    texture_transform       : [f32; 4], // (x, y), (scale_x, scale_y) / color (r, g, b, a)
    mask                    : [f32; 4], // (x, y), (scale_x, scale_y)
    border_radius           : [f32; 4], // (top_left, top_right, bottom_right, bottom_left)
    border_radius_mask      : [f32; 4], // (top_left, top_right, bottom_right, bottom_left)
    data0                   : [f32; 4], // (depth, rotation, unused, unused)
    meta0                   : [f32; 4], // (type, tex_layer, unused, unused)
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]  #[repr(C)]
struct RendererGPUData {
    data0           : [f32; 4], // aspect_ratio, unused, unused, unused
}

struct RendererFrameConfig {
    num_items           : usize,
    depth               : f32,
    texture             : Option<(f32, f32, f32, f32)>,
    color               : (f32, f32, f32, f32),
    mask                : (f32, f32, f32, f32),    
    border_radius       : (f32, f32, f32, f32),
    border_radius_mask  : (f32, f32, f32, f32),
    frame_res           : (f32, f32),
}

pub struct Renderer{
    pipeline        : crate::GraphicsPipeline,   
    gpu_data        : crate::UniformBuffer<RendererGPUData>,
    items           : Vec<RendererItem>,
    items_gpu       : crate::ShaderStorageBuffer,
    frame_config    : RendererFrameConfig,
    target_view     : Option<wgpu::TextureView>,
}

impl Default for RendererGPUData {
    fn default() -> Self {
        Self {
            data0       : [1.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Default for RendererFrameConfig {
    fn default() -> Self {
        Self {
            num_items           : 0,
            depth               : 0.0,
            texture             : None,
            color               : (0.0, 0.0, 0.0, 1.0),
            mask                : (0.0, 0.0, 1.0, 1.0),
            border_radius       : (0.0, 0.0, 0.0, 0.0),
            border_radius_mask  : (0.0, 0.0, 0.0, 0.0),
            frame_res           : (800.0, 600.0),
        }
    }
}


impl Renderer{
    
    const MAX_ITEMS: usize = 1024 * 64;
    
    
    pub async fn new(instance: &crate::GPUInstance, target_format: wgpu::TextureFormat) -> Result<Self, String> {
        
        let gpu_data = crate::UniformBuffer::<RendererGPUData>::new(instance, Some("RendererGPUData")).await?;
        let items_gpu = crate::ShaderStorageBuffer::new(instance, Self::MAX_ITEMS * std::mem::size_of::<RendererItem>(), Some("RendererItems")).await?;
        let pipeline = crate::GraphicsPipeline::new(instance,
            include_str!("../assets/shaders/renderer.wgsl").to_string(),
            target_format,
            vec![
            &gpu_data.layout(),
            &items_gpu.layout(),
            ],
            Some("MainRendererPipeline")).await?;
            
            let obj = Self {
                pipeline        : pipeline,
                gpu_data        : gpu_data,
                items           : vec![],
                items_gpu       : items_gpu,
                frame_config    : RendererFrameConfig::default(),
                target_view     : None,
            };
            
            Ok(obj)
        }
        
        pub fn begin(&mut self) -> Result<(), String> {
            self.items.clear();
            self.frame_config = RendererFrameConfig::default();
            self.target_view = None;
            
            Ok(())
        }
        
        pub fn flush(&mut self, instance: &crate::GPUInstance) -> Result<(), String> {
            self.frame_config.num_items = self.items.len();
            self.items_gpu.upload(instance, bytemuck::cast_slice(&self.items))?;
            self.items.clear();

            // flush here
            if self.target_view.is_none() {
                return Err("No target view!".to_string());
            }

            self.gpu_data.upload(instance);

            let mut encoder = instance.encoder("Renderer::flush.encoder");
            let target_view = self.target_view.as_ref().unwrap();
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear( wgpu::Color {r: 0.0, g: 0.0, b: 0.0, a: 0.0}),
                            store:  wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                label: Some("Renderer::flush.rpass"),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            rpass.set_pipeline(&self.pipeline.pipeline());
            rpass.set_bind_group(0, &self.gpu_data.bind_group(), &[]);
            rpass.set_bind_group(1, &self.items_gpu.bind_group(), &[]);
            rpass.draw(0..6, 0..self.frame_config.num_items as u32);
            drop(rpass);

            instance.submit(encoder);

            Ok(())
        }

        pub fn set_target_view(&mut self, view: wgpu::TextureView) {
            self.target_view = Some(view);
        }
        
        
        pub fn end(&mut self, instance: &crate::GPUInstance) -> Result<(), String> {
            self.flush(instance)?;
            Ok(())
        }
        
        pub fn set_frame_res(&mut self, frame_res: (u32, u32)) {
            self.frame_config.frame_res = (frame_res.0 as f32, frame_res.1 as f32);
            self.gpu_data.data_mut().data0[0] = frame_res.0 as f32 / frame_res.1 as f32;
        }
        
        pub fn set_color(&mut self, color_r: f32, color_g: f32, color_b: f32, color_a: f32) {
            self.frame_config.color = (color_r, color_g, color_b, color_a);
            self.frame_config.texture = None;
        }
        
        pub fn set_depth(&mut self, depth: f32) {
            self.frame_config.depth = depth;
        }
        
        pub fn set_mask(&mut self, mask_x: f32, mask_y: f32, mask_w: f32, mask_h: f32) {
            self.frame_config.mask = (mask_x, mask_y, mask_w, mask_h);
        }
        
        pub fn set_maskp(&mut self, mask_x: i32, mask_y: i32, mask_w: i32, mask_h: i32) {
            let mask_x = mask_x as f32 / self.frame_config.frame_res.0;
            let mask_y = mask_y as f32 / self.frame_config.frame_res.1;
            let mask_w = mask_w as f32 / self.frame_config.frame_res.0;
            let mask_h = mask_h as f32 / self.frame_config.frame_res.1;
            
            self.set_mask(mask_x, mask_y, mask_w, mask_h);
        }

        pub fn set_mask_border_radius(&mut self, top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) {
            self.frame_config.border_radius_mask = (top_left, top_right, bottom_right, bottom_left);
        }
        
        pub fn set_border_radius(&mut self, top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) {
            self.frame_config.border_radius = (top_left, top_right, bottom_right, bottom_left);
        }
        
        pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
            let mut item = RendererItem {
                offset_scale        : [x, y, w, h],
                texture_transform   : [0.0, 0.0, 0.0, 0.0],
                mask                : [self.frame_config.mask.0, self.frame_config.mask.1, self.frame_config.mask.2, self.frame_config.mask.3],
                border_radius       : [self.frame_config.border_radius.0, self.frame_config.border_radius.1, self.frame_config.border_radius.2, self.frame_config.border_radius.3],
                border_radius_mask  : [self.frame_config.border_radius_mask.0, self.frame_config.border_radius_mask.1, self.frame_config.border_radius_mask.2, self.frame_config.border_radius_mask.3],
                data0               : [self.frame_config.depth, 0.0, 0.0, 0.0],
                meta0               : [0.0, 0.0, 0.0, 0.0],
            };
            
            // update color
            item.texture_transform = [self.frame_config.color.0, self.frame_config.color.1, self.frame_config.color.2, self.frame_config.color.3];
            
            self.items.push(item);
        }
        
        pub fn rectp(&mut self, x: i32, y: i32, w: i32, h: i32) {
            let x = x as f32 / self.frame_config.frame_res.0;
            let y = y as f32 / self.frame_config.frame_res.1;
            let w = w as f32 / self.frame_config.frame_res.0;
            let h = h as f32 / self.frame_config.frame_res.1;
            
            self.rect(x, y, w, h)
        }
        
        
    }