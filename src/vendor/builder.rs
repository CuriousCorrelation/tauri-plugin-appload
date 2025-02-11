use super::{Result, VendorConfig};

#[derive(Default, Clone)]
pub struct VendorConfigBuilder {
    content: Option<Vec<u8>>,
}

impl VendorConfigBuilder {
    pub fn new() -> Self {
        Self { content: None }
    }

    pub fn content(mut self, content: Vec<u8>) -> Self {
        self.content = Some(content);
        self
    }

    pub fn build(self) -> Result<VendorConfig> {
        Ok(VendorConfig {
            content: self.content,
        })
    }
}
