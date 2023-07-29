use std::fmt::Debug;

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Node<T: PartialOrd + Clone> {
    leaf: bool,
    count: usize,
    keys: Vec<T>,
    children: Vec<Box<Node<T>>>,
    t: usize,
}

#[allow(dead_code)]
impl<T: PartialOrd + Clone> Node<T> {
    fn empty(t: usize) -> Self {
        return Node {
            keys: Vec::with_capacity(t),
            children: Vec::with_capacity(t + 1),
            count: 0,
            leaf: false,
            t,
        };
    }

    fn leaf(t: usize) -> Self {
        return Node {
            keys: Vec::with_capacity(t),
            children: Vec::with_capacity(t + 1),
            count: 0,
            leaf: true,
            t,
        };
    }

    fn is_full(&self, index: usize) -> bool {
        self.children[index].count == 2 * self.t - 1
    }

    fn is_empty(&self) -> bool {
        return self.count == 0;
    }


    /**
     * return siblings of node with given index
     * first element is given node
     */
    fn siblings(children: &mut Vec<Box<Node<T>>>, index: usize) -> Vec<Option<&mut Box<Node<T>>>> {
        let mut result: Vec<_> = children
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i == index || *i + 1 == index || *i == index + 1)
            .map(|(_, v)| Some(v))
            .collect();

        if result.len() == 3 {
            return result
        }

        if index == 0 {
            result.insert(0, None);

            return result
        }

        result.push(None);

        result
    }

    fn to_vec(&self) -> Vec<T> {
        if self.leaf {
            return self.keys.clone();
        }

        let mut acc: Vec<T> = vec![];

        for i in 0..=self.count {
            acc.append(&mut self.children[i].to_vec());
            if i != self.count {
                acc.push(self.keys[i].clone());
            }
        }

        acc
    }

    fn contains(&self, value: T) -> bool {
        let mut i = 0;

        while i < self.count {
            if self.keys[i] >= value {
                break;
            }

            i += 1
        }

        if self.leaf {
            if i == self.count {
                return false;
            }

            return self.keys[i] == value;
        }

        if i == self.count {
            return self.children[i].contains(value);
        }

        if value == self.keys[i] {
            return true;
        }

        self.children[i].contains(value)
    }

    /**
     * self is nonfull node
     * self.children[i] is full node
     */
    fn split(&mut self, i: usize) {
        let left = &mut self.children[i];
        let right = &mut Box::new(Node::<T>::empty(self.t));

        right.leaf = left.leaf;
        right.count = self.t - 1;

        right.keys = left.keys.split_off(self.t);

        if !left.leaf {
            right.children = left.children.split_off(self.t)
        }

        left.count = self.t - 1;

        self.keys.insert(i, left.keys.pop().unwrap());
        self.children.insert(i + 1, right.clone());
        self.count += 1;
    }

    fn insert_nonfull(&mut self, value: T) {
        if self.leaf {
            let mut i = self.count;

            self.keys.push(value.clone());
            self.count += 1;

            while i >= 1 && value < self.keys[i - 1] {
                self.keys.swap(i, i - 1);
                i -= 1;
            }

            return;
        }

        let mut i = self.count - 1;

        while i > 0 && self.keys[i] >= value {
            i -= 1;
        }

        if self.keys[i] < value {
            i += 1;
        }

        if self.is_full(i) {
            self.split(i);
            if value > self.keys[i] {
                i += 1
            }
        }

        self.children[i].insert_nonfull(value);
    }

    fn remove_key(&mut self, key: T) -> bool {
        let result = self.keys.iter().position(|element| *element == key);

        match result {
            None => false,
            Some(index) => {
                self.keys.remove(index);

                return true;
            }
        }
    }

    /**
     * merge target node with one sibling
     * returns index of new leaf
     */
    fn merge(children: &mut Vec<Box<Node<T>>>, target: usize, delimeter_value: T) -> usize {
        let mut siblings = Self::siblings(children, target);
        let target_leaf = siblings.remove(1).unwrap();

        let possible_left = siblings[0].as_mut();

        if let Some(left) = possible_left {
            left.keys.push(delimeter_value);
            left.keys.append(&mut target_leaf.keys);

            children.remove(target);

            return target - 1;
        }

        let possible_right = siblings[1].as_mut();

        if let Some(right) = possible_right {
            target_leaf.keys.push(delimeter_value);
            target_leaf.keys.append(&mut right.keys);

            children.remove(target + 1);

            return target
        }

        panic!("Unable to merge index: {}", target)
    }

    fn delete(&mut self, value: T) -> bool {
        let mut i = 0;

        while i < self.count && value > self.keys[i] {
            i += 1;
        }

        if i == self.count || self.keys[i] != value {
            let max_value = self.keys[i - 1].clone();
            let target_leaf = &mut self.children[i];

            if target_leaf.leaf {
                return self.delete_from_leaf(value, i)
            }

            let operation_status = target_leaf.delete(value);

            if target_leaf.count < self.t {
                Self::merge(&mut self.children, i, max_value);
            }

            return operation_status
        }

        let moved_to_node = &mut self.children[i + 1];

        self.keys[i] = moved_to_node.keys.remove(0);
        moved_to_node.keys.insert(0, value.clone());

        if moved_to_node.leaf {
            return self.delete_from_leaf(value, i + 1)
        }

        let operation_status = moved_to_node.delete(value);
        let delimeter_value = self.keys[i].clone();

        if moved_to_node.count < self.t {
            Self::merge(&mut self.children, i, delimeter_value);
        }

        operation_status
    }

    /**
     * self refers to root of leaf
     * i is index of child, where we want to remove value
     * returns status of operation: did element remove
     */
    fn delete_from_leaf(&mut self, value: T, i: usize) -> bool {
        let mut siblings = Self::siblings(&mut self.children, i);
        let target_leaf = siblings.remove(1).unwrap();

        if target_leaf.count >= self.t {
            return target_leaf.remove_key(value);
        }

        let possible_left = siblings[0].as_mut();

        if let Some(left) = possible_left {
            if left.count >= self.t {
                let max_value = left.keys.pop().unwrap();
                let delimeter_value = self.keys.remove(i);

                target_leaf.keys.insert(0, delimeter_value);
                self.keys.insert(i, max_value);

                return true;
            }
        }

        let possible_right = siblings[1].as_mut();

        if let Some(right) = possible_right {
            if right.count >= self.t {
                let min_value = right.keys.remove(0);
                let delimeter_value = self.keys.remove(i);

                target_leaf.keys.insert(0, delimeter_value);
                self.keys.insert(i, min_value);

                return true;
            }
        }

        let new_leaf_index = Self::merge(&mut self.children, i, self.keys[i].clone());
        let status = self.children[new_leaf_index].remove_key(value);

        if !status {
            return false
        }

        true
    }
}

struct BTree<T: PartialOrd + Clone> {
    root: Box<Node<T>>,
    t: usize,
}

impl<T: PartialOrd + Clone> BTree<T> {
    pub fn new(t: usize) -> BTree<T> {
        let tree = BTree {
            root: Box::new(Node::<T>::leaf(t)),
            t,
        };

        tree
    }

    fn insert(&mut self, value: T) {
        let current_root = &mut self.root;

        if current_root.count != 2 * self.t - 1 {
            current_root.insert_nonfull(value);

            return;
        }

        let new_root = Box::new(Node::<T> {
            count: 0,
            keys: vec![],
            children: vec![current_root.clone()],
            leaf: false,
            t: self.t,
        });

        self.root = new_root;

        self.root.split(0);
        self.root.insert_nonfull(value)
    }

    fn delete(&mut self, value: T) -> bool {
        if self.root.leaf {
            return self.root.remove_key(value);
        }
        return self.root.delete(value);
    }

    fn to_vec(&self) -> Vec<T> {
        return self.root.to_vec();
    }

    fn contains(&self, value: T) -> bool {
        return self.root.contains(value);
    }
}
fn main() {
    let mut tree = BTree::<i32>::new(3);

    let arr: Vec<i32> = vec![1, 2, -1, 2, 5, 100, -10, 0, 124, 4];

    for v in arr.iter() {
        tree.insert(*v)
    }

    tree.delete(1);

    println!("{:?}", tree.root);
}
