use {
    pollster::FutureExt,
    std::ops::Deref,
    vello::{peniko::Color, wgpu, Scene},
    winit::{dpi::PhysicalSize, window::Window as OsWindow},
};

/// The list of texture formats that are supported by the `vello` renderer.
const SUPPORTED_FORMATS: &[wgpu::TextureFormat] = &[
    wgpu::TextureFormat::Rgba8Unorm,
    wgpu::TextureFormat::Bgra8Unorm,
];

/// Contains the ,state required to leverage the GPU for rendering the UI.
pub struct Renderer {
    /// The graphics API instance.
    ///
    /// This is used to create the device and the surface.
    instance: wgpu::Instance,

    /// The GPU adapter that has been selected for rendering.
    adapter: wgpu::Adapter,

    /// The output format of the renderer. This is the format that the surfaces created with the
    /// renderer must support.
    output_format: wgpu::TextureFormat,

    /// An open connection with the graphics processing unit.
    ///
    /// This device is associated with the `adapter` and is used to create resources such
    /// as buffers, images, and shaders.
    device: wgpu::Device,

    /// The queue used to submit commands to the GPU.
    queue: wgpu::Queue,

    /// The 2D renderer used to draw the UI.
    renderer: vello::Renderer,
}

impl Renderer {
    /// Creates a new [`Renderer`] instance.
    ///
    /// This function will create a new instance of the renderer and initialize the graphics API
    /// for rendering the UI.
    ///
    /// # Returns
    ///
    /// The renderer must be created along with a [`SurfaceTarget`] that represents a window to
    /// be rendered to.
    pub fn new_with_surface(window: OsWindow) -> (Self, SurfaceTarget) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::empty(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let surface = SurfaceTarget::new(&instance, window);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface.surface),
            })
            .block_on()
            .expect("Found no suitable GPU adapter for rendering");

        let output_format = *surface
            .surface
            .get_capabilities(&adapter)
            .formats
            .iter()
            .find(|format| SUPPORTED_FORMATS.contains(format))
            .expect("No supported format found for the created surface");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Yadaw UI Renderer"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .block_on()
            .expect("Failed to establish a connection with the GPU");

        let renderer = vello::Renderer::new(
            &device,
            vello::RendererOptions {
                surface_format: Some(output_format),
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: None,
            },
        )
        .expect("Failed to create the 2D renderer");

        let renderer = Self {
            instance,
            adapter,
            output_format,
            device,
            queue,
            renderer,
        };

        (renderer, surface)
    }

    /// Creates a new [`SurfaceTarget`].
    pub fn create_surface(&self, window: OsWindow) -> SurfaceTarget {
        let surface = SurfaceTarget::new(&self.instance, window);

        assert!(
            surface
                .surface
                .get_capabilities(&self.adapter)
                .formats
                .contains(&self.output_format),
            "The surface does not support the output format of the renderer",
        );

        surface
    }
}

/// Contains both a surface and the target window it is rendering to.
pub struct SurfaceTarget {
    /// The surface that is responsible for rendering to the associated `window` target.
    ///
    /// # Safety
    ///
    /// This surface is bound to the lifetime of the window it is rendering to. It is important that
    /// `surface` is dropped before `window` for that reason.
    ///
    /// The drop order ensures that this is the case. This means that those two fields must not
    /// be reordered.
    surface: wgpu::Surface<'static>,

    /// The window on which the surface is rendered.
    window: OsWindow,
}

impl SurfaceTarget {
    /// Creates a new [`SurfaceTarget`] instance.
    ///
    /// This function will create a new surface that is bound to the specified window.
    fn new(instance: &wgpu::Instance, window: OsWindow) -> Self {
        unsafe {
            let target = wgpu::SurfaceTargetUnsafe::from_window(&window)
                .expect("Failed to create a surface");

            let surface = instance
                .create_surface_unsafe(target)
                .expect("Failed to create a surface");

            Self { surface, window }
        }
    }

    /// Re-configures the swapchain of the surface to match the new size.
    pub fn re_configure_swapchain(&self, renderer: &Renderer, size: PhysicalSize<u32>) {
        self.surface.configure(
            &renderer.device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: renderer.output_format,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 1,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
            },
        );
    }

    /// Renders the content of the window.
    pub fn render(
        &self,
        renderer: &mut Renderer,
        size: PhysicalSize<u32>,
        clear_color: Color,
        scene: &Scene,
    ) {
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("Failed to get the current texture");

        assert!(!surface_texture.suboptimal, "The surface is suboptimal");

        renderer
            .renderer
            .render_to_surface(
                &renderer.device,
                &renderer.queue,
                scene,
                &surface_texture,
                &vello::RenderParams {
                    base_color: clear_color,
                    width: size.width,
                    height: size.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .expect("Failed to render the scene");

        self.window.pre_present_notify();
        surface_texture.present();
    }
}

impl Deref for SurfaceTarget {
    type Target = OsWindow;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.window
    }
}
