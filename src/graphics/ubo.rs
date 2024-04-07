pub struct UniformBuffer<T> 
    where T: Copy + Clone + Default + bytemuck::Pod + bytemuck::Zeroable
{
    buffer          : wgpu::Buffer,
    bind_group      : wgpu::BindGroup,
    layout          : wgpu::BindGroupLayout,
    data            : T,
    _phantom        : std::marker::PhantomData<T>,
}

impl<T> UniformBuffer<T> 
    where T: Copy + Clone + Default + bytemuck::Pod + bytemuck::Zeroable
{
    pub async fn new(instance: &crate::GPUInstance, label: Option<&str>) -> Result<Self, String> {
        let buffer = instance.device().create_buffer(&wgpu::BufferDescriptor {
            label               : label.clone(),
            size                : std::mem::size_of::<T>() as u64,
            usage               : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation  : false,
        });

        let layout = instance.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label       : label.clone(),
            entries     : &[wgpu::BindGroupLayoutEntry {
                binding     : 0,
                visibility  : wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty          : wgpu::BindingType::Buffer {
                                ty                  : wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset  : false, 
                                min_binding_size    : None,
                            },
                count       : None,
            }],
        });

        let bind_group = instance.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label       : label.clone(),
            layout      : &layout,
            entries     : &[wgpu::BindGroupEntry {
                binding     : 0,
                resource    : buffer.as_entire_binding(),
            }],
        });

        let obj = Self {
            buffer          : buffer,
            bind_group      : bind_group,
            layout          : layout,
            data            : T::default(),
            _phantom        : std::marker::PhantomData,
        };

        obj.upload(instance);

        Ok(obj)
    }

    pub fn upload(&self, instance: &crate::GPUInstance) {
        instance.queue().write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data]));
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn set_data(&mut self, data: T) {
        self.data = data;
    }

    pub fn size(&self) -> u64 {
        std::mem::size_of::<T>() as u64
    }

}