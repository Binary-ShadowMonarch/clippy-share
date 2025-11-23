pub struct Net;

impl Net {
    pub async fn start() {
        println!("[stub] net started");
    }

    pub async fn broadcast(&self, _data: Vec<u8>) {
        println!("[stub] broadcasting {} bytes", _data.len());
    }
}
