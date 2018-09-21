/// Adaptive Radix Tree (non-concurrent)
///
/// Important notes: nodes 48 must have
/// pointers initialized to u8::MAX.
use std::fmt;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr::null_mut;

#[derive(Clone, Debug)]
pub struct Art<T> {
    root: *mut Node<T>,
}

impl<T> Default for Art<T>
where
    T: fmt::Debug,
{
    fn default() -> Art<T> {
        let root = Node::default();
        let root_ptr = Box::into_raw(Box::new(root));

        Art { root: root_ptr }
    }
}

impl<T> Art<T>
where
    T: fmt::Debug,
{
    pub fn set(&mut self, k: Vec<u8>, v: T) {
        unsafe { (*self.root).set(k, v) }
    }

    pub fn get<'a>(&self, k: &'a [u8]) -> Option<&'a T> {
        unsafe { (*self.root).get(k) }
    }
}

impl<T> Deref for Art<T> {
    type Target = Node<T>;

    fn deref(&self) -> &Node<T> {
        unsafe { &*self.root }
    }
}

impl<T> DerefMut for Art<T> {
    fn deref_mut(&mut self) -> &mut Node<T> {
        unsafe { &mut *self.root }
    }
}

#[derive(Clone)]
pub enum Node<T> {
    Node4 {
        value: Option<T>,
        prefix: Vec<u8>,
        index: [u8; 4],
        pointers: [*mut Node<T>; 4],
    },
    Node16 {
        value: Option<T>,
        prefix: Vec<u8>,
        index: [u8; 16],
        pointers: [*mut Node<T>; 16],
    },
    Node48 {
        value: Option<T>,
        prefix: Vec<u8>,
        index: [u8; 256],
        pointers: [*mut Node<T>; 48],
    },
    Node256 {
        value: Option<T>,
        prefix: Vec<u8>,
        pointers: [*mut Node<T>; 256],
    },
}

use Node::*;

impl<T> Default for Node<T>
where
    T: fmt::Debug,
{
    fn default() -> Node<T> {
        Node4 {
            value: None,
            prefix: vec![],
            index: [255; 4],
            pointers: [null_mut(); 4],
        }
    }
}

impl<T> fmt::Debug for Node<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node4 {
                value,
                prefix,
                index,
                pointers,
            } => write!(
                f,
                "Node4 {{ value: {:?}, prefix: {:?}, index: {:?}, pointers: {:?} }}",
                value, prefix, index, pointers
            ),
            Node16 {
                value,
                prefix,
                index,
                pointers,
            } => write!(
                f,
                "Node16 {{ value: {:?}, prefix: {:?}, index: {:?}, pointers: {:?} }}",
                value, prefix, index, pointers
            ),
            Node48 { value, prefix, .. } => write!(
                f,
                "Node48 {{ value: {:?}, prefix: {:?}, index: OMITTED, pointers: OMITTED }}",
                value, prefix,
            ),
            Node256 { value, prefix, .. } => {
                write!(f, "Node256 {{ value: {:?}, prefix: {:?}, OMITTED }}", value, prefix,)
            }
        }
    }
}

impl<T> Index<usize> for Node<T>
where
    T: fmt::Debug,
{
    type Output = *mut Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            Node4 { ref pointers, .. } => &pointers[index],
            Node16 { ref pointers, .. } => &pointers[index],
            Node48 { ref pointers, .. } => &pointers[index],
            Node256 { ref pointers, .. } => &pointers[index],
        }
    }
}

impl<T> IndexMut<usize> for Node<T>
where
    T: fmt::Debug,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            Node4 {
                ref mut pointers, ..
            } => &mut pointers[index],
            Node16 {
                ref mut pointers, ..
            } => &mut pointers[index],
            Node48 {
                ref mut pointers, ..
            } => &mut pointers[index],
            Node256 {
                ref mut pointers, ..
            } => &mut pointers[index],
        }
    }
}

impl<T> Node<T>
where
    T: fmt::Debug,
{
    pub fn set(&mut self, key: Vec<u8>, value: T) {
        self.insert(&*key, 0, value);
    }

    fn insert(&mut self, key: &[u8], mut depth: usize, value: T) {
        println!(
            "key: {:?} depth: {} prefix: {:?}",
            key,
            depth,
            self.prefix()
        );
        if self.prefix() == &key[depth..] {
            self.set_value(value);
            return;
        }

        let common_prefix_len =
            common_prefix_len(&*key, self.prefix());

        // prefix mismatch, create a new parent for the current node
        if common_prefix_len != self.prefix().len() {
            println!("splitting prefix: ");
            let common_prefix =
                self.prefix()[..common_prefix_len].to_vec();

            let old_byte = self.prefix()[common_prefix_len];
            let old_prefix =
                self.prefix()[common_prefix_len + 1..].to_vec();
            let new_byte = key[common_prefix_len];
            let new_prefix = key[common_prefix_len + 1..].to_vec();

            println!("old_byte: {}, old: {:?}", old_byte, old_prefix);
            println!("common: {:?}", common_prefix);
            println!("before: {:?}", self);

            let mut node = Node::default();
            node.set_prefix(common_prefix);

            std::mem::swap(self, &mut node);

            node.set_prefix(old_prefix);

            println!("after old: {:?}", node);

            self.add_child(old_byte, node);

            let mut new_node = Node::default();
            new_node.set_prefix(new_prefix);
            new_node.set_value(value);
            self.add_child(new_byte, new_node);
            println!("after self: {:?}", self);
            return;
        }

        depth += self.prefix().len();

        println!(".");
        if let Some(next_idx) = self.find_child(key[depth]) {
            let ptr = self[next_idx];
            unsafe { (*ptr).insert(key, depth + 1, value) }
        } else {
            if self.is_full() {
                self.grow();
            }

            let new_node = Node4 {
                value: Some(value),
                prefix: key[depth + 1..].to_vec(),
                index: [0u8; 4],
                pointers: [null_mut(); 4],
            };

            println!("added child at byte {}", key[depth]);
            self.add_child(key[depth], new_node);
        }
    }

    fn set_prefix(&mut self, p: Vec<u8>) {
        match self {
            Node4 { ref mut prefix, .. }
            | Node16 { ref mut prefix, .. }
            | Node48 { ref mut prefix, .. }
            | Node256 { ref mut prefix, .. } => *prefix = p,
        }
    }

    fn set_value(&mut self, v: T) {
        match self {
            Node4 { ref mut value, .. }
            | Node16 { ref mut value, .. }
            | Node48 { ref mut value, .. }
            | Node256 { ref mut value, .. } => *value = Some(v),
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&T> {
        if !key.starts_with(self.prefix()) {
            return None;
        }
        let skip = self.prefix().len();

        if skip == key.len() {
            return self.value();
        }

        let child_idx = match self.find_child(key[skip]) {
            Some(c) => c,
            None => return None,
        };

        let child = self[child_idx];

        unsafe {
            match *child {
                ref next_node => next_node.get(&key[skip + 1..]),
            }
        }
    }

    fn value(&self) -> Option<&T> {
        match self {
            Node4 {
                value: Some(ref v), ..
            }
            | Node16 {
                value: Some(ref v), ..
            }
            | Node48 {
                value: Some(ref v), ..
            }
            | Node256 {
                value: Some(ref v), ..
            } => Some(v),
            _ => None,
        }
    }

    fn is_full(&self) -> bool {
        match self {
            Node4 { ref pointers, .. } => {
                pointers.iter().all(|p| !p.is_null())
            }
            Node16 { ref pointers, .. } => {
                pointers.iter().all(|p| !p.is_null())
            }
            Node48 { ref pointers, .. } => {
                pointers.iter().all(|p| !p.is_null())
            }
            Node256 { ref pointers, .. } => {
                pointers.iter().all(|p| !p.is_null())
            }
        }
    }

    fn add_child(&mut self, byte: u8, child: Node<T>) {
        let ptr = Box::into_raw(Box::new(child));

        match self {
            Node4 {
                index, pointers, ..
            } => {
                let idx = pointers
                    .iter()
                    .position(|p| p.is_null())
                    .expect("node must not be empty");
                index[idx] = byte;
                pointers[idx] = ptr;
            }
            Node16 {
                index, pointers, ..
            } => {
                let idx = pointers
                    .iter()
                    .position(|p| p.is_null())
                    .expect("node must not be empty");
                index[idx] = byte;
                pointers[idx] = ptr;
            }
            Node48 {
                index, pointers, ..
            } => {
                let idx = pointers
                    .iter()
                    .position(|p| p.is_null())
                    .expect("node must not be empty");
                index[byte as usize] = idx as u8;
                pointers[idx] = ptr;
            }
            Node256 { pointers, .. } => {
                if !pointers[byte as usize].is_null() {
                    panic!("replacing existing node");
                }

                pointers[byte as usize] = ptr;
            }
        }
    }

    fn grow(&mut self) {
        let new = match self {
            Node4 {
                value,
                index,
                pointers,
                prefix,
            } => {
                let old = index
                    .iter()
                    .cloned()
                    .zip(pointers.iter().cloned());

                let mut index = [0u8; 16];
                let mut pointers = [null_mut(); 16];

                for (i, (byte, ptr)) in old.enumerate() {
                    index[i] = byte;
                    pointers[i] = ptr;
                }

                Node16 {
                    value: value.take(),
                    prefix: prefix.clone(),
                    index: index,
                    pointers: pointers,
                }
            }
            Node16 {
                value,
                prefix,
                index,
                pointers,
            } => {
                let old = index
                    .iter()
                    .cloned()
                    .zip(pointers.iter().cloned());

                let mut index = [0u8; 256];
                let mut pointers = [null_mut(); 48];

                for (i, (byte, ptr)) in old.enumerate() {
                    index[byte as usize] = i as u8;
                    pointers[i] = ptr;
                }

                Node48 {
                    value: value.take(),
                    prefix: prefix.clone(),
                    index: index,
                    pointers: pointers,
                }
            }
            Node48 {
                value,
                prefix,
                index,
                pointers,
            } => {
                let old = index.iter().enumerate().filter_map(
                    |(byte, idx)| {
                        if *idx < 48 {
                            Some((
                                byte as u8,
                                pointers[*idx as usize],
                            ))
                        } else {
                            None
                        }
                    },
                );

                let mut pointers = [null_mut(); 256];
                for (byte, ptr) in old {
                    pointers[byte as usize] = ptr;
                }

                Node256 {
                    value: value.take(),
                    prefix: prefix.clone(),
                    pointers: pointers,
                }
            }
            Node256 { .. } => panic!("tried to grow a Node256"),
        };

        *self = new;
    }

    fn prefix(&self) -> &[u8] {
        match self {
            Node4 { ref prefix, .. }
            | Node16 { ref prefix, .. }
            | Node48 { ref prefix, .. }
            | Node256 { ref prefix, .. } => &*prefix,
        }
    }

    /// get index for searched byte
    fn find_child(&self, byte: u8) -> Option<usize> {
        match self {
            Node4 {
                ref index,
                ref pointers,
                ..
            } => {
                for (i, b) in index.iter().enumerate() {
                    if *b == byte && !pointers[i].is_null() {
                        return Some(i as usize);
                    }
                }
                None
            }
            Node16 {
                ref index,
                ref pointers,
                ..
            } => {
                // TODO SSE
                for (i, b) in index.iter().enumerate() {
                    if *b == byte && !pointers[i].is_null() {
                        return Some(i as usize);
                    }
                }
                None
            }
            Node48 {
                ref index,
                ref pointers,
                ..
            } => {
                let i = index[byte as usize];

                if i >= 48 {
                    // idx does not point to valid slot
                    None
                } else {
                    assert_ne!(
                        null_mut(),
                        pointers[i as usize],
                        "should not have a null pointer with a valid index"
                    );
                    Some(i as usize)
                }
            }
            Node256 { .. } => Some(byte as usize),
        }
    }
}

fn common_prefix_len(a: &[u8], b: &[u8]) -> usize {
    for (i, (ae, be)) in a.iter().zip(b.iter()).enumerate() {
        if ae != be {
            return i;
        }
    }
    std::cmp::min(a.len(), b.len())
}

#[test]
fn test_common_prefix_len() {
    assert_eq!(common_prefix_len(b"abc", b"abc"), 3);
    assert_eq!(common_prefix_len(b"ab", b"abc"), 2);
    assert_eq!(common_prefix_len(b"abc", b"ab"), 2);
    assert_eq!(common_prefix_len(b"bc", b"abc"), 0);
    assert_eq!(common_prefix_len(b"abc", b"bc"), 0);
}
