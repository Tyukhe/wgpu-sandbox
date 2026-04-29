pub struct Render {}

impl Render {
    pub async fn new() -> anyhow::Result<Self> {
        log::info!("Render initialized");
        Ok(Self {})
    }
}
