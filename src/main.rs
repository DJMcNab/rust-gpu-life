use spirv_builder::{CompileResult, SpirvBuilder};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use futures::executor::block_on;

use rust_gpu_life::State;

fn main() {
    env_logger::init();
    let (shadertx, shaderrx) = std::sync::mpsc::sync_channel::<CompileResult>(0);
    let (rebuildtx, rebuildrx) = std::sync::mpsc::sync_channel(1);
    let _ = std::thread::spawn(move || loop {
        rebuildrx.recv().unwrap();
        let compilation_result = SpirvBuilder::new("./shaders", "spirv-unknown-vulkan1.2")
            .capability(spirv_builder::Capability::Int8)
            .print_metadata(false)
            .build();
        match compilation_result {
            Ok(res) => shadertx.send(res).unwrap(),
            Err(err) => println!("Shader compilation failed with error {}", err),
        }
    });
    rebuildtx.send(()).unwrap();
    let initial_shader = shaderrx.recv().unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    // Since main can't be async, we're going to need to block
    let mut state = block_on(State::new(&window, initial_shader));
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        state.resize(**new_inner_size);
                    }

                    _ => {}
                }
            }
        }
        Event::RedrawRequested(_) => {
            match shaderrx.try_recv() {
                Ok(res) => state.update_pipelines(res),
                Err(err) => match err {
                    std::sync::mpsc::TryRecvError::Empty => (),
                    std::sync::mpsc::TryRecvError::Disconnected => {
                        panic!("Background building thread broke")
                    }
                },
            }
            state.update();
            match state.render() {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SwapChainError::Lost) => state.recreate_swap_chain(),
                // The system is out of memory, we should probably quit
                Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        _ => {}
    });
}
