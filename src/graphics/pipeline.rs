pub struct GraphicsPipeline {
    layout          : wgpu::PipelineLayout,
    pipeline        : wgpu::RenderPipeline,
}

impl GraphicsPipeline {

    pub async fn new(instance: &crate::GPUInstance, shader_source: String, target_format: wgpu::TextureFormat, bind_grp_layouts: Vec<&wgpu::BindGroupLayout>, label: Option<&str>) -> Result<Self, String> {

        let shader = instance.device().create_shader_module(wgpu::ShaderModuleDescriptor {
            label       : label,
            source      : wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let layout = instance.device().create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label                   : label.clone(),
            bind_group_layouts      : &bind_grp_layouts,
            push_constant_ranges    : &[],
        });

        let pipeline = instance.device().create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: label,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module          : &shader,
                entry_point     : "vs_main",
                buffers         : &[],
            },
            fragment: Some(wgpu::FragmentState {
                module          : &shader,
                entry_point     : "fs_main",
                targets         : &[Some(wgpu::ColorTargetState {
                                    format      : target_format,
                                    blend       : Some(wgpu::BlendState::ALPHA_BLENDING),
                                    write_mask  : wgpu::ColorWrites::ALL,
                                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology            : wgpu::PrimitiveTopology::TriangleList,
                strip_index_format  : None,
                front_face          : wgpu::FrontFace::Ccw,
                cull_mode           : None,
                polygon_mode        : wgpu::PolygonMode::Fill,
                unclipped_depth     : false,
                conservative        : false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count                      : 1,
                mask                       : !0,
                alpha_to_coverage_enabled  : false,
            },
            multiview: None,
        });

        let obj = Self {
            layout          : layout,
            pipeline        : pipeline,
        };

        Ok(obj)
    }

    pub fn layout(&self) -> &wgpu::PipelineLayout {
        &self.layout
    }

    pub fn pipeline(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }

}