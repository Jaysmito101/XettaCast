pub struct ShaderStorageBuffer {
    buffer              : wgpu::Buffer,
    layout              : wgpu::BindGroupLayout,
    bind_group          : wgpu::BindGroup,
    size                : usize,
}

impl ShaderStorageBuffer {

    pub async fn new(instance: &crate::GPUInstance, size: usize, label: Option<&str>) -> Result<Self, String> {
        let buffer = instance.device().create_buffer(&wgpu::BufferDescriptor {
            label               : label,
            size                : size as u64,
            usage               : wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation  : false,
        });

        let layout = instance.device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: label,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { 
                        ty                  : wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset  : false,
                        min_binding_size    : None
                    },
                    count: None,
                }
            ],
        });

        let bind_group = instance.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: label,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
        });

        Ok(Self {
            buffer          : buffer,
            layout          : layout,
            bind_group      : bind_group,
            size            : size,
        })
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn upload(&self, instance: &crate::GPUInstance, data: &[u8]) -> Result<(), String> {
        instance.queue().write_buffer(&self.buffer, 0, data);
        Ok(())
    }


}