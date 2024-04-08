#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]  #[repr(C)]
struct RendererItem {
    offset_scale            : [f32; 4], // (x, y), (scale_x, scale_y)
    texture_transform       : [f32; 4], // (x, y), (scale_x, scale_y) / color (r, g, b, a)
    meta0                   : [f32; 4], // (type, tex_layer, unused, unused)
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]  #[repr(C)]
struct RendererGPUData {
    color           : [f32; 4],
}

pub struct Renderer{
    pipeline        : crate::GraphicsPipeline,   
    gpu_data        : crate::UniformBuffer<RendererGPUData>,
    items           : Vec<RendererItem>,
}

impl Default for RendererGPUData {
    fn default() -> Self {
        Self {
            color       : [0.0, 0.0, 0.0, 1.0],
        }
    }
}

impl Renderer{
    pub async fn new(instance: &crate::GPUInstance, target_format: wgpu::TextureFormat) -> Result<Self, String> {

        let gpu_data = crate::UniformBuffer::<RendererGPUData>::new(instance, Some("RendererGPUData")).await?;
        let pipeline = crate::GraphicsPipeline::new(instance,
            include_str!("../assets/shaders/renderer.wgsl").to_string(),
            target_format,
            vec![
                &gpu_data.layout(),
            ],
            Some("MainRendererPipeline")).await?;

        let obj = Self {
            pipeline        : pipeline,
            gpu_data        : gpu_data,
            items           : vec![],
        };
        
        Ok(obj)
    }

    pub fn begin(&mut self) -> Result<(), String> {
        self.items.clear();

        Ok(())
    }

    fn flush_internal<'a>(&'a self, mut rpass: wgpu::RenderPass<'a>, instance: &crate::GPUInstance) -> Result<wgpu::RenderPass<'a>, String> {

        let pipeline = self.pipeline.pipeline();
        
        rpass.set_pipeline(&pipeline);
        rpass.set_bind_group(0, &self.gpu_data.bind_group(), &[]);
        rpass.draw(0..6, 0..1);


        Ok(rpass)
    }

    pub fn flush<'a>(&'a mut self, render_pass: wgpu::RenderPass<'a>, instance: &crate::GPUInstance) -> Result<wgpu::RenderPass<'a>, String> {

        let rpass = self.flush_internal(render_pass, instance)?;

        Ok(rpass)
    }


    pub fn end(&mut self, instance: &crate::GPUInstance) -> Result<(), String> {
        self.gpu_data.data_mut().color[0] =  if self.gpu_data.data_mut().color[0] > 1.0 {
            0.0
        } else {
            self.gpu_data.data_mut().color[0] + 0.0001
        };
        self.gpu_data.upload(instance);

        Ok(())
    }

}