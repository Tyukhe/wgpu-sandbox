use wgpu_project::run;

fn main() {
    env_logger::init();
    if let Err(e) = run() {
        log::error!("{}", e);
    }
}
