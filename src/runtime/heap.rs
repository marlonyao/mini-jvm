use crate::runtime::frame::Value;
use std::collections::HashMap;

/// A simple object representation for the heap.
#[derive(Debug, Clone)]
pub struct JObject {
    pub class_name: String,
    pub fields: HashMap<String, Value>,
    /// For String objects, store the string value directly
    pub string_value: Option<String>,
}

/// Simple heap using a vector with index-based references.
pub struct Heap {
    objects: Vec<Option<JObject>>,
    free_list: Vec<usize>,
    /// Array data for int/byte/short/char/long/float/double arrays
    array_data: HashMap<usize, Vec<Value>>,
    /// Array data for reference arrays
    array_refs: HashMap<usize, Vec<Value>>,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            objects: Vec::new(),
            free_list: Vec::new(),
            array_data: HashMap::new(),
            array_refs: HashMap::new(),
        }
    }

    /// Allocate a new object on the heap, returns its index.
    pub fn alloc(&mut self, class_name: String) -> usize {
        let obj = JObject {
            class_name,
            fields: HashMap::new(),
            string_value: None,
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

    /// Allocate a String object with a given value.
    pub fn alloc_string(&mut self, value: String) -> usize {
        let idx = self.alloc("java/lang/String".to_string());
        self.objects[idx].as_mut().unwrap().string_value = Some(value);
        idx
    }

    /// Get the string value of an object (if it's a String).
    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.objects.get(index)
            .and_then(|o| o.as_ref())
            .and_then(|o| o.string_value.as_deref())
    }

    // --- Array support ---

    /// Allocate a new array on the heap.
    pub fn alloc_array(&mut self, class_name: String, length: usize) -> usize {
        let mut obj = JObject {
            class_name,
            fields: HashMap::new(),
            string_value: None,
        };
        // Store array data in special fields
        obj.fields.insert("$length".to_string(), Value::I32(length as i32));
        obj.fields.insert("$data".to_string(), Value::Null); // marker
        // Store actual array data separately
        let idx = if let Some(free_idx) = self.free_list.pop() {
            self.objects[free_idx] = Some(obj);
            free_idx
        } else {
            let idx = self.objects.len();
            self.objects.push(Some(obj));
            idx
        };
        // Initialize array elements (stored in a side table)
        self.array_data.insert(idx, vec![Value::I32(0); length]);
        self.array_refs.insert(idx, vec![Value::Null; length]);
        idx
    }

    /// Get the length of an array.
    pub fn get_array_length(&self, index: usize) -> usize {
        self.array_data.get(&index).map(|a| a.len()).unwrap_or(0)
    }

    /// Get an int element from an array.
    pub fn get_array_int(&self, index: usize, element_index: usize) -> i32 {
        self.array_data.get(&index)
            .and_then(|a| a.get(element_index))
            .map(|v| v.as_i32())
            .unwrap_or(0)
    }

    /// Set an int element in an array.
    pub fn set_array_int(&mut self, index: usize, element_index: usize, value: i32) {
        if let Some(a) = self.array_data.get_mut(&index) {
            if element_index < a.len() {
                a[element_index] = Value::I32(value);
            }
        }
    }

    /// Get a reference element from an array.
    pub fn get_array_ref(&self, index: usize, element_index: usize) -> Value {
        self.array_refs.get(&index)
            .and_then(|a| a.get(element_index))
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// Set a reference element in an array.
    pub fn set_array_ref(&mut self, index: usize, element_index: usize, value: Value) {
        if let Some(a) = self.array_refs.get_mut(&index) {
            if element_index < a.len() {
                a[element_index] = value;
            }
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
