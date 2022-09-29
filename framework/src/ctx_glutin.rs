// creates opengl context
pub fn create_window(width: u32, height: u32, title: &str) -> Result<glutin::Window, glutin::CreationError>
{
    glutin::WindowBuilder::new()
        .with_dimensions(width, height)
        .with_title(title)
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (4, 5)))
        .with_gl_profile(glutin::GlProfile::Core)
        .build()
}

pub fn load_gl_from<T: glutin::GlContext>(context: &T)
{
    unsafe
    {
        context.make_current().unwrap();
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);
    }
}
