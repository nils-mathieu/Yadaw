use {
    pollster::FutureExt,
    std::cell::Cell,
    vello::{peniko, wgpu},
    winit::{dpi::PhysicalSize, window::Window},
};

/// Returns whether the provided format is supported by the `vello` renderer.
fn is_format_supported_by_vello(format: wgpu::TextureFormat) -> bool {
    use wgpu::TextureFormat::*;

    matches!(format, Rgba8Unorm | Bgra8Unorm)
}

/// Contains the state required to render the UI (mainly GPU resources).
pub struct Renderer {
    /// The instance.
    ///
    /// This will be used when creating new surfaces.
    instance: wgpu::Instance,
    /// The GPU adapter that has been selected to render the UI.
    adapter: wgpu::Adapter,

    /// An open connection to the GPU device.
    device: wgpu::Device,
    /// The queue that will be used to submit GPU commands.
    queue: wgpu::Queue,

    /// The output texture format used when rendering the final image to the screen.
    output_format: wgpu::TextureFormat,
    /// The `vello` renderer responsible actually doing the heavy lifting.
    vello_renderer: vello::Renderer,
}

impl Renderer {
    /// Creates a new [`Renderer`] along with a window.
    pub fn new_for_window(window: Box<dyn Window>) -> (Self, WindowAndSurface) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = WindowAndSurface::from_instance(&instance, window);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface.surface),
                force_fallback_adapter: false,
            })
            .block_on()
            .unwrap_or_else(|| panic!("Failed to find a suitable GPU adapter"));

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    ..Default::default()
                },
                None,
            )
            .block_on()
            .unwrap_or_else(|err| panic!("Failed to create device: {err}"));

        let output_format = *surface
            .surface
            .get_capabilities(&adapter)
            .formats
            .iter()
            .find(|&&f| is_format_supported_by_vello(f))
            .unwrap();

        let vello_renderer = vello::Renderer::new(&device, vello::RendererOptions {
            surface_format: Some(output_format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::area_only(),
            num_init_threads: None,
        })
        .unwrap_or_else(|err| panic!("Failed to create the 2D renderer: {err}"));

        (
            Self {
                instance,
                adapter,
                device,
                queue,
                output_format,
                vello_renderer,
            },
            surface,
        )
    }
}

/// Represents a window and its associated surface.
pub struct WindowAndSurface {
    /// The surface object.
    ///
    /// # Safety
    ///
    /// This field must be dropped *before* the window because it references it.
    surface: wgpu::Surface<'static>,

    /// The winit window.
    window: Box<dyn Window>,

    /// The cached size of the window/surface.
    size: Cell<PhysicalSize<u32>>,
    /// The present mode to be used when presenting the surface.
    present_mode: Cell<wgpu::PresentMode>,
    /// Whether the surface is dirty and needs to be redrawn.
    surface_dirty: Cell<bool>,
    /// The color to use when clearing the surface.
    base_color: Cell<peniko::Color>,
}

impl WindowAndSurface {
    /// Creates a new [`WindowAndSurface`] object.
    ///
    /// This function does not check whether the created surface supports a specific format.
    fn from_instance(instance: &wgpu::Instance, window: Box<dyn Window>) -> Self {
        let surface = unsafe {
            let target = wgpu::SurfaceTargetUnsafe::from_window(&window)
                .unwrap_or_else(|err| panic!("Failed to create surface: {err}"));
            instance
                .create_surface_unsafe(target)
                .unwrap_or_else(|err| panic!("Failed to create surface: {err}"))
        };

        let size = window.surface_size();

        Self {
            surface,
            window,
            size: Cell::new(size),
            present_mode: Cell::new(wgpu::PresentMode::AutoVsync),
            surface_dirty: Cell::new(true),
            base_color: Cell::new(peniko::Color::BLACK),
        }
    }

    /// Creates a new [`WindowAndSurface`] object.
    ///
    /// This function will panic if the created surface does not support the output
    /// format of the renderer.
    pub fn new(renderer: &Renderer, window: Box<dyn Window>) -> Self {
        let surface = Self::from_instance(&renderer.instance, window);

        assert!(
            surface
                .surface
                .get_capabilities(&renderer.adapter)
                .formats
                .contains(&renderer.output_format),
            "The surface does not support the output format of the renderer",
        );

        surface
    }

    /// Notifies the window that its size has changed.
    #[inline]
    pub fn set_size(&self, new_size: PhysicalSize<u32>) {
        self.size.set(new_size);
        self.surface_dirty.set(true);
    }

    /// Returns the cached size of the window/surface.
    #[inline]
    pub fn cached_size(&self) -> PhysicalSize<u32> {
        self.size.get()
    }

    /// Sets the present mode to be used when presenting the surface.
    #[inline]
    pub fn set_present_mode(&self, present_mode: wgpu::PresentMode) {
        self.present_mode.set(present_mode);
        self.surface_dirty.set(true);
    }

    /// Sets the base color to use when clearing the surface.
    #[inline]
    pub fn set_base_color(&self, color: peniko::Color) {
        self.base_color.set(color);
    }

    /// Renders the provided scene to the surface.
    pub fn render(&self, renderer: &mut Renderer, scene: &vello::Scene) {
        let size = self.size.get();

        if size.width == 0 || size.height == 0 {
            return;
        }

        if self.surface_dirty.replace(false) {
            self.surface
                .configure(&renderer.device, &wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: renderer.output_format,
                    width: size.width,
                    height: size.height,
                    present_mode: self.present_mode.get(),
                    desired_maximum_frame_latency: 1,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: vec![],
                });
        }

        let frame = self
            .surface
            .get_current_texture()
            .unwrap_or_else(|err| panic!("Failed to get the next surface frame: {err}"));
        debug_assert!(!frame.suboptimal, "The surface frame is suboptimal");

        renderer
            .vello_renderer
            .render_to_surface(
                &renderer.device,
                &renderer.queue,
                scene,
                &frame,
                &vello::RenderParams {
                    base_color: self.base_color.get(),
                    width: size.width,
                    height: size.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap_or_else(|err| panic!("Failed to render to surface: {err}"));

        self.window.pre_present_notify();
        frame.present();
    }

    /// Returns a reference to the winit window.
    #[inline]
    pub fn winit_window(&self) -> &dyn Window {
        self.window.as_ref()
    }
}
