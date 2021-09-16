pub struct Application {
    pub(crate) identifier: String, // max len 8
    pub(crate) authentication_code: [u8; 3],
    pub(crate) data: Vec<u8>,
}

impl Application {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn authentication_code(&self) -> &[u8] {
        &self.authentication_code
    }
}
