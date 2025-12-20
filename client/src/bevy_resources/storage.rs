#[derive(Debug, Clone)]
pub struct Storage<V> {
    storage: Vec<V>,
}

impl<V> Storage<V> {
    pub fn new(values: Vec<V>) -> Storage<V> {
        Storage { storage: values }
    }

    pub fn add(&mut self, v: V) {
        assert!(self.storage.len() < usize::MAX);

        self.storage.push(v);
    }

    pub fn get_by_id(&self, id: usize) -> Option<&V> {
        self.storage.get(id)
    }
}
