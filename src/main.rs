#[macro_use]
extern crate gfx;

extern crate gfx_window_glutin;
extern crate glutin;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;
use glutin::{GlContext, GlRequest};
use glutin::Api::OpenGl;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

mod window;

const BLACK: [f32; 4] = [0.3, 0.3, 0.3, 1.0];

// Put this code above your main function
gfx_defines!{
    vertex Vertex {
        pos: [f32; 4] = "a_Pos",
        color: [f32; 3] = "a_Color",
    }

    constant Transform {
        transform: [[f32; 4];4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        transform: gfx::ConstantBuffer<Transform> = "Transform",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

pub fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let windowbuilder = glutin::WindowBuilder::new()
        .with_title("Volumetric Data".to_string())
        .with_dimensions(512, 512);
    let contextbuilder = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(OpenGl,(3,2)))
        .with_vsync(true);
    let (window, mut device, mut factory, color_view, mut depth_view) =
        gfx_glutin::init::<ColorFormat, DepthFormat>(windowbuilder, contextbuilder, &events_loop);

    let pso = factory.create_pipeline_simple(
        include_bytes!("window/render/quad.vert"),
        include_bytes!("window/render/quad.frag"),
        pipe::new()
    ).unwrap();

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    const TRIANGLE: [Vertex; 6] = [
        Vertex { pos: [ -0.2, -0.5, 0.0, 1.0 ], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [ -0.8, -0.5, 0.0, 1.0 ], color: [0.0, 1.0, 0.0] },
        Vertex { pos: [ -0.5,  0.5, 0.0, 1.0 ], color: [0.0, 0.0, 1.0] },
        Vertex { pos: [  0.2, -0.5, 0.0, 1.0 ], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [  0.8, -0.5, 0.0, 1.0 ], color: [0.0, 1.0, 0.0] },
        Vertex { pos: [  0.5,  0.5, 0.0, 1.0 ], color: [0.0, 0.0, 1.0] },
    ];
    //Identity Matrix
    const TRANSFORM: Transform = Transform {
        transform: [[1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]]
    };

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
    let transform_buffer = factory.create_constant_buffer(1);
    let data = pipe::Data {
        vbuf: vertex_buffer,
        transform: transform_buffer,
        out: color_view.clone(),
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::Closed |
                    glutin::WindowEvent::KeyboardInput {
                        input: glutin::KeyboardInput {
                            virtual_keycode: Some(glutin::VirtualKeyCode::Escape), ..
                        }, ..
                    } => running = false,
                    _ => {}
                }
            }
        });

        encoder.clear(&color_view, BLACK); //clear the framebuffer with a color(color needs to be an array of 4 f32s, RGBa)
        encoder.update_buffer(&data.transform, &[TRANSFORM], 0); //update buffers
        encoder.draw(&slice, &pso, &data); // draw commands with buffer data and attached pso
        encoder.flush(&mut device); // execute draw commands

        window.swap_buffers().unwrap();
        device.cleanup();
    }
}