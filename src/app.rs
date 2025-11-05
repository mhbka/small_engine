use crate::state::State;

pub struct App<'a> {
    #[cfg(target_arch = "wasm32")]
    pub proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    pub state: Option<State<'a>>,
}

impl<'a> App<'a> {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &winit::event_loop::EventLoop<State>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}