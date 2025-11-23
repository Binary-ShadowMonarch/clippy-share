pub struct CryptoEngine;

impl CryptoEngine {
    pub fn new() -> Self {
        Self
    }
    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        println!("[stub] encrypt {} bytes", data.len());
        data.to_vec()
    }
    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        println!("[stub] decrypt {} bytes", data.len());
        data.to_vec()
    }
}
