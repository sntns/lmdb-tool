use std::fmt;

#[derive(Clone)]
pub struct Element {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key_s: String = self.key.iter().map(|&c| c as char).collect();
        let data_s: String = self.value.iter().map(|&c| c as char).collect();
        f.debug_struct("Element")
            .field("key", &key_s)
            .field("value", &data_s)
            .finish()
    }
}
