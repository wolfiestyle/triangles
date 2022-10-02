use glutin::dpi::PhysicalSize;
use glutin::event_loop::EventLoop;
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, PossiblyCurrent, WindowedContext};

pub struct GlutinWindow {
    pub event_loop: EventLoop<()>,
    pub context: WindowedContext<PossiblyCurrent>,
}

impl GlutinWindow {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        let el = EventLoop::new();

        let wb = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(width, height))
            .with_title(title);

        let ctx = ContextBuilder::new()
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)))
            .with_gl_profile(glutin::GlProfile::Core)
            .build_windowed(wb, &el)
            .unwrap();

        let ctx = unsafe { ctx.make_current().unwrap() };

        GlutinWindow {
            event_loop: el,
            context: ctx,
        }
    }

    pub fn load_gl(&self) {
        gl::load_with(|symbol| self.context.get_proc_address(symbol));
    }
}
