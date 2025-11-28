fn main() {
    glazer::run(
        proa::Memory::default(),
        1280,
        720,
        proa::handle_input,
        proa::update_and_render,
        glazer::debug_target(),
    );
}
