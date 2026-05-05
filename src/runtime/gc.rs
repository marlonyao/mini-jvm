/// Mark-Sweep Garbage Collector for mini-jvm.
///
/// Algorithm:
///   1. Mark phase: trace all reachable objects from GC roots
///      GC roots = thread stack frames (locals + operand stack) + static fields
///   2. Sweep phase: free all unmarked objects
///
/// Complexity: O(V + E) where V = objects, E = reference edges

use crate::runtime::frame::Value;
use crate::runtime::heap::Heap;
use std::collections::HashSet;

/// Run a full garbage collection cycle.
/// Returns (freed_count, remaining_count).
pub fn gc(thread_stack: &[crate::runtime::Frame], heap: &mut Heap) -> (usize, usize) {
    let marked = mark(thread_stack, heap);
    let freed = sweep(heap, &marked);
    let remaining = heap.live_count();
    (freed, remaining)
}

/// Mark phase: find all reachable objects.
fn mark(thread_stack: &[crate::runtime::Frame], heap: &Heap) -> HashSet<usize> {
    let mut marked: HashSet<usize> = HashSet::new();
    let mut worklist: Vec<usize> = Vec::new();

    // --- Collect GC roots ---

    // Root 1: References in thread stack frames (locals + operand stack)
    for frame in thread_stack {
        collect_refs_from_values(&frame.locals, &mut worklist);
        collect_refs_from_values(&frame.operand_stack, &mut worklist);
    }

    // Root 2: Static fields that hold references
    for value in heap.static_fields.values() {
        if let Value::Object(idx) = value {
            if *idx < heap.objects.len() && heap.objects[*idx].is_some() {
                worklist.push(*idx);
            }
        }
    }

    // --- Trace references (BFS) ---
    while let Some(idx) = worklist.pop() {
        if marked.contains(&idx) {
            continue; // already visited
        }
        if idx >= heap.objects.len() {
            continue; // invalid index
        }
        if heap.objects[idx].is_none() {
            continue; // already freed
        }

        marked.insert(idx);

        // Trace: object instance fields
        if let Some(obj) = heap.get(idx) {
            for value in obj.fields.values() {
                if let Value::Object(field_idx) = value {
                    if !marked.contains(field_idx) {
                        worklist.push(*field_idx);
                    }
                }
            }
        }

        // Trace: reference array elements
        if let Some(arr) = heap.array_refs.get(&idx) {
            for value in arr {
                if let Value::Object(elem_idx) = value {
                    if !marked.contains(elem_idx) {
                        worklist.push(*elem_idx);
                    }
                }
            }
        }
    }

    marked
}

/// Extract Object references from a slice of Values.
fn collect_refs_from_values(values: &[Value], worklist: &mut Vec<usize>) {
    for value in values {
        if let Value::Object(idx) = value {
            worklist.push(*idx);
        }
    }
}

/// Sweep phase: free all unmarked objects and their associated arrays.
fn sweep(heap: &mut Heap, marked: &HashSet<usize>) -> usize {
    let mut freed = 0;
    let total = heap.objects.len();

    // Collect indices to free (can't modify while iterating)
    let mut to_free: Vec<usize> = Vec::new();
    for idx in 0..total {
        if heap.objects[idx].is_some() && !marked.contains(&idx) {
            to_free.push(idx);
        }
    }

    // Free unmarked objects
    for idx in &to_free {
        heap.objects[*idx] = None;
        heap.free_list.push(*idx);
        heap.array_data.remove(idx);
        heap.array_refs.remove(idx);
        freed += 1;
    }

    freed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::frame::Frame;
    use crate::runtime::heap::Heap;

    fn make_frame(locals: Vec<Value>, stack: Vec<Value>) -> Frame {
        let mut frame = Frame::new(locals.len().max(1), vec![]);
        frame.locals = locals;
        frame.operand_stack = stack;
        frame
    }

    #[test]
    fn test_gc_no_roots() {
        let mut heap = Heap::new();
        heap.alloc("A".to_string());
        heap.alloc("B".to_string());
        // No frames → nothing is reachable → both freed
        let (freed, remaining) = gc(&[], &mut heap);
        assert_eq!(freed, 2);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_gc_root_in_local() {
        let mut heap = Heap::new();
        let idx0 = heap.alloc("A".to_string());
        let _idx1 = heap.alloc("B".to_string()); // unreachable

        let frame = make_frame(vec![Value::Object(idx0)], vec![]);
        let (freed, remaining) = gc(&[frame], &mut heap);

        assert_eq!(freed, 1); // B freed
        assert_eq!(remaining, 1); // A alive
        assert!(heap.get(idx0).is_some());
    }

    #[test]
    fn test_gc_root_in_stack() {
        let mut heap = Heap::new();
        let idx = heap.alloc("A".to_string());
        heap.alloc("B".to_string()); // unreachable

        let frame = make_frame(vec![Value::I32(0)], vec![Value::Object(idx)]);
        let (freed, remaining) = gc(&[frame], &mut heap);

        assert_eq!(freed, 1);
        assert_eq!(remaining, 1);
    }

    #[test]
    fn test_gc_field_tracing() {
        let mut heap = Heap::new();
        let inner = heap.alloc("Inner".to_string());
        let outer = heap.alloc("Outer".to_string());
        heap.get_mut(outer).unwrap().fields.insert("ref".to_string(), Value::Object(inner));
        heap.alloc("Garbage".to_string()); // unreachable

        // Only outer is a root, but inner should be kept via field reference
        let frame = make_frame(vec![Value::Object(outer)], vec![]);
        let (freed, remaining) = gc(&[frame], &mut heap);

        assert_eq!(freed, 1); // Garbage
        assert_eq!(remaining, 2); // Outer + Inner
    }

    #[test]
    fn test_gc_array_tracing() {
        let mut heap = Heap::new();
        let ref1 = heap.alloc("A".to_string());
        let ref2 = heap.alloc("B".to_string());
        let arr = heap.alloc_array("[Ljava/lang/Object;".to_string(), 2);
        heap.set_array_ref(arr, 0, Value::Object(ref1));
        heap.set_array_ref(arr, 1, Value::Object(ref2));
        heap.alloc("Garbage".to_string()); // unreachable

        let frame = make_frame(vec![Value::Object(arr)], vec![]);
        let (freed, remaining) = gc(&[frame], &mut heap);

        assert_eq!(freed, 1); // Garbage
        assert_eq!(remaining, 3); // arr + A + B
    }

    #[test]
    fn test_gc_static_fields() {
        let mut heap = Heap::new();
        let idx = heap.alloc("Global".to_string());
        heap.set_static_field("App.instance".to_string(), Value::Object(idx));
        heap.alloc("Garbage".to_string());

        let (freed, remaining) = gc(&[], &mut heap);

        assert_eq!(freed, 1); // Garbage
        assert_eq!(remaining, 1); // Global kept by static field
    }

    #[test]
    fn test_gc_circular_reference() {
        let mut heap = Heap::new();
        let a = heap.alloc("A".to_string());
        let b = heap.alloc("B".to_string());
        // A.ref → B, B.ref → A (circular)
        heap.get_mut(a).unwrap().fields.insert("ref".to_string(), Value::Object(b));
        heap.get_mut(b).unwrap().fields.insert("ref".to_string(), Value::Object(a));

        // Both unreachable → both freed
        let (freed, remaining) = gc(&[], &mut heap);
        assert_eq!(freed, 2);
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_gc_circular_with_root() {
        let mut heap = Heap::new();
        let a = heap.alloc("A".to_string());
        let b = heap.alloc("B".to_string());
        let c = heap.alloc("C".to_string()); // unreachable
        heap.get_mut(a).unwrap().fields.insert("ref".to_string(), Value::Object(b));
        heap.get_mut(b).unwrap().fields.insert("ref".to_string(), Value::Object(a));

        let frame = make_frame(vec![Value::Object(a)], vec![]);
        let (freed, remaining) = gc(&[frame], &mut heap);

        assert_eq!(freed, 1); // C
        assert_eq!(remaining, 2); // A + B (circular but reachable)
    }

    #[test]
    fn test_gc_string_object() {
        let mut heap = Heap::new();
        let s = heap.alloc_string("hello".to_string());
        heap.alloc("Garbage".to_string());

        let frame = make_frame(vec![Value::Object(s)], vec![]);
        let (freed, remaining) = gc(&[frame], &mut heap);

        assert_eq!(freed, 1);
        assert_eq!(remaining, 1);
        assert_eq!(heap.get_string(s), Some("hello"));
    }

    #[test]
    fn test_gc_keeps_null_slots() {
        let mut heap = Heap::new();
        // Free a slot, then GC — freed slot should stay free
        let idx = heap.alloc("A".to_string());
        heap.free(idx);
        heap.alloc("B".to_string());

        let (freed, remaining) = gc(&[], &mut heap);
        assert_eq!(freed, 1); // B is unreachable
        assert_eq!(remaining, 0);
    }

    #[test]
    fn test_gc_multiple_frames() {
        let mut heap = Heap::new();
        let a = heap.alloc("A".to_string());
        let b = heap.alloc("B".to_string());
        let c = heap.alloc("C".to_string()); // unreachable

        let frame1 = make_frame(vec![Value::Object(a)], vec![]);
        let frame2 = make_frame(vec![Value::Object(b)], vec![]);
        let (freed, remaining) = gc(&[frame1, frame2], &mut heap);

        assert_eq!(freed, 1); // C
        assert_eq!(remaining, 2); // A + B
    }
}
