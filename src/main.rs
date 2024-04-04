#[tokio::main]
async fn main() -> Result<(), String> {
    std::panic::set_hook(Box::new(|panic_info| {
        xettacast::utils::die("Panic", &format!("{:?}", panic_info));
    }));

    env_logger::builder().filter_level(log::LevelFilter::Debug).init();
    let event_loop = winit::event_loop::EventLoop::new();
    let mut app = xettacast::App::new(&event_loop).await.unwrap();

    event_loop.run(move |event, _, control_flow| {
        app.global_update().unwrap();
        match event {
            winit::event::Event::MainEventsCleared => {
                app.request_redraw();
            },
            winit::event::Event::WindowEvent { event, window_id: _ } => {
                app.on_event(event);
            },
            winit::event::Event::RedrawRequested(_) => {
                if !app.on_update().unwrap() {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
            },
            _ => {}
        }

    });
}
