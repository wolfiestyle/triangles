use fw::{Buffer, Framebuffer, Program, Shader, TexFramebuffer, Texture2d, UniformValue, VertexArray};
use glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::ControlFlow;
use rand::Rng;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::Instant;

// shared GL state
struct GlState {
    tex_img: Texture2d,
    program: Program,
    fbo: TexFramebuffer,
    mse: TexMse,
}

// buffer with random triangles
struct TriangleBuf {
    vao: VertexArray,
    vbo: Rc<Buffer<f32>>,
    n_verts: usize,
    _mse: Cell<Option<f32>>,
}

const ELEMS_PER_VERT: usize = 6;

impl TriangleBuf {
    fn random(n_tris: usize) -> Self {
        let n_elems = n_tris * 3 * ELEMS_PER_VERT;
        let mut rng = rand::thread_rng();
        let data: Vec<f32> = (0..n_elems).map(|_| rng.gen()).collect();
        let vbo = Buffer::new(gl::DYNAMIC_DRAW, &data);
        TriangleBuf::from_vbo(vbo)
    }

    fn from_vbo(vbo: Buffer<f32>) -> Self {
        let mut vao = VertexArray::new();
        let n_verts = vbo.len() / ELEMS_PER_VERT;
        let vbo_rc = Rc::new(vbo);
        vao.set_attribute(0, vbo_rc.clone(), 2, 0, ELEMS_PER_VERT); // position: vec2
        vao.set_attribute(1, vbo_rc.clone(), 4, 2, ELEMS_PER_VERT); // color: vec4
        TriangleBuf {
            vao: vao,
            vbo: vbo_rc,
            n_verts: n_verts,
            _mse: Cell::new(None),
        }
    }

    fn draw(&self, gl_state: &GlState) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Enable(gl::BLEND);
        }
        gl_state.program.set_active();
        self.vao.draw(gl::TRIANGLES, 0, self.n_verts as u32);
        unsafe { gl::Disable(gl::BLEND) };
    }

    // evaluated more than once, so we need to cache this
    fn calc_mse(&self, gl_state: &GlState) -> f32 {
        if let Some(val) = self._mse.get() {
            val
        } else {
            gl_state.fbo.bind();
            //unsafe{ gl::Viewport(0, 0, TEX_SIZE as i32, TEX_SIZE as i32) };
            self.draw(&gl_state);
            let val = gl_state.mse.run(&gl_state.tex_img, gl_state.fbo.get_tex());
            self._mse.set(Some(val));
            val
        }
    }

    // mutate a single number in the array
    fn mutate(&self) -> OldState {
        let elem_id = rand::thread_rng().gen_range(0..self.vbo.len());
        let old_elem = self.vbo.get(elem_id);
        self.vbo.set(elem_id, rand::random());
        self._mse.set(None);
        OldState {
            id: elem_id,
            val: old_elem,
        }
    }

    fn revert(&self, st: OldState) {
        self.vbo.set(st.id, st.val);
        self._mse.set(None);
    }
}

struct OldState {
    id: usize,
    val: f32,
}

// helper to draw a textured quad to screen
struct TexDraw {
    vao: VertexArray,
    program: Program,
}

impl TexDraw {
    fn new() -> Self {
        let prog = Program::new(&[
            Shader::new(gl::VERTEX_SHADER, &[include_str!("tex.vert.glsl")]).unwrap(),
            Shader::new(gl::FRAGMENT_SHADER, &[include_str!("tex.frag.glsl")]).unwrap(),
        ])
        .unwrap();

        let coords = Rc::new(Buffer::new(gl::STATIC_DRAW, &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0f32]));

        let mut vao = VertexArray::new();
        vao.set_attribute(0, coords, 2, 0, 2);

        TexDraw { vao: vao, program: prog }
    }

    fn draw<T: UniformValue>(&self, tex: T) {
        Framebuffer::unbind();
        //unsafe{ gl::Viewport(0, 0, FB_SIZE as i32, FB_SIZE as i32) };
        self.program.get_uniform("tex").unwrap().set(tex);
        self.program.set_active();
        self.vao.draw(gl::TRIANGLE_STRIP, 0, 4);
    }
}

// calcs the sum if all the texels of a texture
struct TexFold {
    program: Program,
    tex_cache: RefCell<HashMap<(u32, u32), Rc<Texture2d>>>,
}

const WG_SIZE: u32 = 16; // local_size_* from shader

impl TexFold {
    fn new() -> Self {
        let fold_op_src = "vec4 fold_op(vec4 acc, vec4 val) { return acc + val; }";
        let prog = Program::new(&[Shader::new(gl::COMPUTE_SHADER, &[include_str!("fold.comp.glsl"), fold_op_src]).unwrap()]).unwrap();
        prog.get_uniform("src").unwrap().set(0);
        prog.get_uniform("dest").unwrap().set(0);
        TexFold {
            program: prog,
            tex_cache: Default::default(),
        }
    }

    fn run(&self, tex_src: &Texture2d) -> [f32; 4] {
        let size_x = tex_src.get_width();
        let size_y = tex_src.get_height();

        let wg_size2 = WG_SIZE * 2;
        assert!(
            size_x % wg_size2 == 0 && size_y % wg_size2 == 0,
            "tex size must be divisible by {}",
            wg_size2
        );

        let mut wg_x = size_x / wg_size2;
        let mut wg_y = size_y / wg_size2;

        self.program.set_active();

        // first iteration
        let tex_iter1 = self.run_compute(wg_x, wg_y, tex_src);

        // check if it's worth to iterate again
        // with local_size 16, this will only run if tex_size >= 1024
        let tex_out = if wg_x % wg_size2 == 0 && wg_y % wg_size2 == 0 {
            wg_x /= wg_size2;
            wg_y /= wg_size2;

            self.run_compute(wg_x, wg_y, &tex_iter1)
        } else {
            tex_iter1
        };

        let result_data = tex_out.read_data(gl::RGBA).unwrap();

        // fold the (hopefully) tiny result texture into the final value
        result_data.into_iter().fold([0f32; 4], vec4_add)
    }

    fn run_compute(&self, wg_x: u32, wg_y: u32, tex_in: &Texture2d) -> Rc<Texture2d> {
        let tex_out = self.get_cached_tex(wg_x, wg_y);

        tex_in.bind_to(0); // tex unit 0 = src
        tex_out.bind_to_image(0, 0, gl::RGBA32F, gl::WRITE_ONLY); // img unit 0 = dest

        unsafe {
            gl::DispatchCompute(wg_x, wg_y, 1);
            gl::MemoryBarrier(gl::TEXTURE_FETCH_BARRIER_BIT | gl::TEXTURE_UPDATE_BARRIER_BIT);
        }

        tex_out
    }

    fn get_cached_tex(&self, width: u32, height: u32) -> Rc<Texture2d> {
        self.tex_cache
            .borrow_mut()
            .entry((width, height))
            .or_insert_with(|| Rc::new(Texture2d::new(width, height, gl::RGBA32F)))
            .clone()
    }
}

// calcs the difference squared between two textures
struct TexDsq {
    program: Program,
}

impl TexDsq {
    fn new() -> Self {
        let prog = Program::new(&[Shader::new(gl::COMPUTE_SHADER, &[include_str!("dsq.comp.glsl")]).unwrap()]).unwrap();
        prog.get_uniform("src1").unwrap().set(0);
        prog.get_uniform("src2").unwrap().set(1);
        prog.get_uniform("dest").unwrap().set(0);
        TexDsq { program: prog }
    }

    fn run(&self, src1: &Texture2d, src2: &Texture2d, dest: &Texture2d) {
        let wg_x = dest.get_width();
        let wg_y = dest.get_height();

        self.program.set_active();

        src1.bind_to(0); // tex unit 0 = src1
        src2.bind_to(1); // tex unit 1 = src2
        dest.bind_to_image(0, 0, gl::RGBA32F, gl::WRITE_ONLY); // img unit 0 = dest

        unsafe {
            gl::DispatchCompute(wg_x, wg_y, 1);
            gl::MemoryBarrier(gl::TEXTURE_FETCH_BARRIER_BIT | gl::TEXTURE_UPDATE_BARRIER_BIT);
        }
    }
}

// calcs the mean square error between two textures
struct TexMse {
    dsq: TexDsq,
    fold: TexFold,
    tex_dsq: Texture2d,
}

impl TexMse {
    fn new(width: u32, height: u32) -> Self {
        TexMse {
            dsq: TexDsq::new(),
            fold: TexFold::new(),
            tex_dsq: Texture2d::new(width, height, gl::RGBA32F),
        }
    }

    fn run(&self, src1: &Texture2d, src2: &Texture2d) -> f32 {
        self.dsq.run(src1, src2, &self.tex_dsq);
        // supposedly you have to divide by the total here, but we don't need to do it
        let mse = self.fold.run(&self.tex_dsq);
        // same here, we sum instead of calculating the average
        mse[0] + mse[1] + mse[2] + mse[3]
    }
}

// maybe i should use a math library
fn vec4_add(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3]]
}

fn vec4_div(a: [f32; 4], b: f32) -> [f32; 4] {
    [a[0] / b, a[1] / b, a[2] / b, a[3] / b]
}

fn main() {
    let mut image_file = String::new();
    let mut output_file = String::new();
    let mut tex_size = 256;
    let mut n_tris = 100;
    let mut draw_interval = 1000;
    {
        use argparse::{ArgumentParser, Store};

        let mut parser = ArgumentParser::new();
        parser.set_description("Approximates an image with random triangles");
        parser
            .refer(&mut image_file)
            .add_argument("image", Store, "Input image file")
            .required();
        parser
            .refer(&mut tex_size)
            .add_option(&["-t", "--tex-size"], Store, "Texture size used in computations");
        parser
            .refer(&mut n_tris)
            .add_option(&["-n", "--num-tris"], Store, "Number of triangles in approximation");
        parser
            .refer(&mut draw_interval)
            .add_option(&["-d", "--draw-interval"], Store, "Display the result after N iterations");
        parser
            .refer(&mut output_file)
            .add_option(&["-o", "--output"], Store, "Output image filename (saved on exit)");
        parser.parse_args_or_exit()
    }

    println!(
        "image: {}\ntexture size: {}\nnum triangles: {}\ndrawing every {} iters",
        image_file, tex_size, n_tris, draw_interval
    );

    // load the reference image
    let img = image::open(image_file).unwrap();

    // init opengl context
    let window = fw::ctx_glutin::GlutinWindow::new(tex_size, tex_size, "triangles");
    fw::ctx_glutin::load_gl_from(&window);
    //fw::enable_debug_callback();

    unsafe {
        gl::Enable(gl::FRAMEBUFFER_SRGB);
        gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ONE);
    }

    // scale and load the image into a texture
    let tex_img = Texture2d::new(tex_size, tex_size, gl::SRGB8_ALPHA8);
    tex_img.load_image(img);

    // we'll use this to draw triangles
    let prog_tris = Program::new(&[
        Shader::new(gl::VERTEX_SHADER, &[include_str!("color.vert.glsl")]).unwrap(),
        Shader::new(gl::FRAGMENT_SHADER, &[include_str!("color.frag.glsl")]).unwrap(),
    ])
    .unwrap();

    // draw to texture setup
    let fb_tex = Texture2d::new(tex_size, tex_size, gl::RGBA8);
    let fbo = fb_tex.into_framebuffer().unwrap();

    // init compute operations
    let texmse = TexMse::new(tex_size, tex_size);

    // set the background color to the average color of the image
    let avg_color = vec4_div(texmse.fold.run(&tex_img), (tex_size * tex_size) as f32);
    unsafe { gl::ClearColor(avg_color[0], avg_color[1], avg_color[2], avg_color[3]) };
    println!("average color: {:?}", avg_color);

    // put all of the above in a struct
    let gl_state = GlState {
        tex_img: tex_img,
        program: prog_tris,
        fbo: fbo,
        mse: texmse,
    };

    // for displaying the results
    let texdraw = TexDraw::new();

    let state = TriangleBuf::random(n_tris);
    let mut best_mse = state.calc_mse(&gl_state);
    let mut iters = 0;

    let start_time = Instant::now();
    let mut frame_time = start_time.clone();
    let mut frame_count = 0;
    let mut frame_total = 0;

    window.event_loop.run(move |ev, _, cf| match ev {
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested,
            ..
        } => {
            *cf = ControlFlow::ExitWithCode(0);
        }
        Event::MainEventsCleared => {
            let old_state = state.mutate();
            let mse = state.calc_mse(&gl_state);
            if mse < best_mse {
                best_mse = mse;
            } else {
                state.revert(old_state);
            }

            frame_count += 1;
            frame_total += 1;
            let elapsed_sec = frame_time.elapsed();
            let elapsed_total = start_time.elapsed();
            if elapsed_sec.as_secs() >= 1 {
                eprint!(
                    "\r{} iters in {:?} ({} iters/s) error: {:?}        ",
                    frame_total, elapsed_total, frame_count, best_mse
                );
                io::stdout().flush().unwrap();

                frame_count = 0;
                frame_time = Instant::now();
            }

            if iters > draw_interval {
                iters = 0;
                window.context.window().request_redraw();
            } else {
                iters += 1;
            }
        }
        Event::RedrawRequested(_) => {
            iters = 0;
            gl_state.fbo.get_tex().bind_to(0);
            texdraw.draw(0);
            window.context.swap_buffers().unwrap();
        }
        Event::LoopDestroyed => {
            if !output_file.is_empty() {
                let tex = gl_state.fbo.get_tex();
                tex.bind_to(0);
                let path = std::path::Path::new(&output_file);
                //FIXME: image is not saved in the correct color space
                let img_data: Vec<u8> = tex.read_data(gl::RGBA).unwrap();
                image::save_buffer(path, &img_data, tex.get_width(), tex.get_height(), image::ColorType::Rgba8).unwrap();
            }
        }
        _ => (),
    });
}
