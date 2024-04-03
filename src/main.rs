use wgpu::util;
use xettacast::utils;

#[tokio::main]
async fn main() -> Result<(), String> {
    std::panic::set_hook(Box::new(|panic_info| {
        xettacast::utils::die("Panic", &format!("{:?}", panic_info));
    }));

    env_logger::builder().filter_level(log::LevelFilter::Debug).init();

    let event_loop = winit::event_loop::EventLoop::new();

    let config = xettacast::ConfigManager::new().await;

    let manager = global_hotkey::GlobalHotKeyManager::new().unwrap();
    manager.register(config.get_trigger().unwrap()).unwrap();


    let window = xettacast::Window::new(&event_loop).await;

    let target_monitor = config.get_monitor().unwrap();

    let monitors = window.get_available_monitors();

    let mut monitor_id = 0;

    event_loop.run(move |event, _, control_flow| {

        match event {
            winit::event::Event::WindowEvent { event, window_id } => {
                match event {
                    winit::event::WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        // log::info!("{:?}", event);
        if let Ok(key) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
            // log::info!("{:?}", key);
            // log::info!("{:?}", window.get_monitor_with_position(mouse_pos.0 as i32, mouse_pos.1 as i32));
            if key.state == global_hotkey::HotKeyState::Pressed {
                monitor_id = (monitor_id + 1) % (monitors.len() + 1);
                if monitor_id == monitors.len() {
                    window.hide();
                }else {
                window.set_monitor(&monitors[monitor_id]);
                }
            }
        }
        
        // log::info!("{:?}", mouse_pos);
    });

    Ok(())    
}
