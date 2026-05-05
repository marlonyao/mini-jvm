use crate::runtime::frame::Value;

/// A simple object representation for the heap.
#[derive(Debug, Clone)]
pub struct JObject {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
}

use std::collections::HashMap;

/// Simple heap using a vector with index-based references.
pub struct Heap {
    objects: Vec<Option<JObject>>,
    free_list: Vec<usize>,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            objects: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Allocate a new object on the heap, returns its index.
    pub fn alloc(&mut self, class_name: String) -> usize {
        let obj = JObject {
            class_name,
            fields: HashMap::new(),
        };

        if let Some(idx) = self.free_list.pop() {
            self.objects[idx] = Some(obj);
            idx
        } else {
            let idx = self.objects.len();
            self.objects.push(Some(obj));
            idx
        }
    }

    /// Get a reference to an object by index.
    pub fn get(&self, index: usize) -> Option<&JObject> {
        self.objects.get(index).and_then(|o| o.as_ref())
    }

    /// Get a mutable reference to an object by index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut JObject> {
        self.objects.get_mut(index).and_then(|o| o.as_mut())
    }

    /// Free an object (mark slot as reusable).
    pub fn free(&mut self, index: usize) {
        if index < self.objects.len() {
            self.objects[index] = None;
            self.free_list.push(index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_alloc_and_get() {
        let mut heap = Heap::new();
        let idx = heap.alloc("java/lang/Object".to_string());
        let obj = heap.get(idx).unwrap();
        assert_eq!(obj.class_name, "java/lang/Object");
    }

    #[test]
    fn test_heap_fields() {
        let mut heap = Heap::new();
        let idx = heap.alloc("MyClass".to_string());
        heap.get_mut(idx).unwrap().fields.insert("x".to_string(), Value::I32(42));
        assert_eq!(heap.get(idx).unwrap().fields.get("x").unwrap().as_i32(), 42);
    }

    #[test]
    fn test_heap_free_and_reuse() {
        let mut heap = Heap::new();
        let idx1 = heap.alloc("A".to_string());
        let idx2 = heap.alloc("B".to_string());
        heap.free(idx1);
        let idx3 = heap.alloc("C".to_string());
        assert_eq!(idx3, idx1); // reused slot
        assert_eq!(heap.get(idx3).unwrap().class_name, "C");
    }
}
