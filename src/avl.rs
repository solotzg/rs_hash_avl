use std::collections::{LinkedList, HashMap};
use std::ptr::NonNull;
use std::cmp::{Ordering, max};
use std::mem;

pub struct Node<K, V> {
    left: Option<NonNull<Node<K, V>>>,
    right: Option<NonNull<Node<K, V>>>,
    parent: Option<NonNull<Node<K, V>>>,
    key: K,
    value: V,
    height: i32,
}

impl<K: Ord, V> Node<K, V> {
    pub unsafe fn node_next(mut node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        match node {
            None => { return None; }
            Some(ptr) => {
                if ptr.as_ref().right.is_none() {
                    loop {
                        let last = node;
                        node = node.unwrap().as_ref().parent;
                        if node.is_none() { break; }
                        if cmp_node_ptr(&node.unwrap().as_ref().left, &last) { break; }
                    }
                } else {
                    node = ptr.as_ref().right;
                    while node.unwrap().as_ref().left.is_some() {
                        node = node.unwrap().as_ref().left;
                    }
                }
            }
        }
        node
    }

    pub unsafe fn node_prev(mut node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        match node {
            None => { return None; }
            Some(ptr) => {
                if ptr.as_ref().left.is_none() {
                    loop {
                        let last = node;
                        node = node.unwrap().as_ref().parent;
                        if node.is_none() { break; }
                        if cmp_node_ptr(&node.unwrap().as_ref().right, &last) { break; }
                    }
                } else {
                    node = ptr.as_ref().left;
                    while node.unwrap().as_ref().right.is_some() {
                        node = node.unwrap().as_ref().right;
                    }
                }
            }
        }
        node
    }
}

pub struct Tree<K: Ord, V: Clone> {
    root: Option<NonNull<Node<K, V>>>,
    count: usize,
}

#[inline]
fn cmp_node_ptr<K, V>(a: &Option<NonNull<Node<K, V>>>, b: &Option<NonNull<Node<K, V>>>) -> bool {
    match *a {
        Some(x) => {
            match *b {
                Some(y) => { x.as_ptr() == y.as_ptr() }
                None => false
            }
        }
        None => { b.is_none() }
    }
}


impl<K: Ord, V: Clone> Tree<K, V> {
    pub fn size(&self) -> usize {
        self.count
    }

    unsafe fn avl_node_first(&self) -> Option<NonNull<Node<K, V>>> {
        let mut node = self.root;
        if node.is_none() {
            return None;
        }
        while node.unwrap().as_ref().left.is_some() {
            node = node.unwrap().as_ref().left;
        }
        node
    }

    unsafe fn avl_node_last(&self) -> Option<NonNull<Node<K, V>>> {
        let mut node = self.root;
        if node.is_none() {
            return None;
        }
        while node.unwrap().as_ref().right.is_some() {
            node = node.unwrap().as_ref().right;
        }
        node
    }

    pub fn new() -> Self {
        Tree { root: None, count: 0 }
    }

    #[inline]
    unsafe fn avl_node_find_link(&mut self, key: K, value: V) -> (Option<NonNull<Node<K, V>>>, Option<(Option<NonNull<Node<K, V>>>, K, V)>) {
        let mut duplicate = None;
        let mut cmp_node = &mut self.root;
        let mut parent = None;
        while let Some(node) = *cmp_node {
            parent = *cmp_node;
            match key.cmp(&node.as_ref().key) {
                Ordering::Less => {
                    cmp_node = &mut (*node.as_ptr()).left;
                }
                Ordering::Equal => {
                    duplicate = *cmp_node;
                    break;
                }
                Ordering::Greater => {
                    cmp_node = &mut (*node.as_ptr()).right;
                }
            }
        }
        if duplicate.is_none() {
            let node_to_insert = Tree::avl_node_link(Box::new(Node {
                left: None,
                right: None,
                parent: None,
                key: key,
                value: value,
                height: 0,
            }), parent, cmp_node);
            assert!(node_to_insert.is_some());
            return (node_to_insert, None);
        }
        (None, Some((duplicate, key, value)))
    }

    #[inline]
    pub fn avl_add_element(&mut self, key: K, value: V) -> bool {
        unsafe {
            self.avl_add_element_with_duplicate(key, value).is_some()
        }
    }

    #[inline]
    pub unsafe fn avl_add_element_with_duplicate(&mut self, key: K, value: V) -> Option<(NonNull<Node<K, V>>, K, V)> {
        let (node_to_insert, res) = self.avl_node_find_link(key, value);
        match res {
            None => {
                self.avl_node_post_insert(node_to_insert);
                self.count += 1;
                return None;
            }
            Some((duplicate, key, value)) => {
                return Some((duplicate.unwrap(), key, value));
            }
        }
    }

    pub fn avl_set_element(&mut self, key: K, value: V) -> Option<V> {
        unsafe {
            match self.avl_add_element_with_duplicate(key, value) {
                None => None,
                Some((mut ptr, _, mut v)) => {
                    mem::swap(&mut ptr.as_mut().value, &mut v);
                    Some(v)
                }
            }
        }
    }


    #[inline]
    unsafe fn avl_node_find(&mut self, what: &K) -> Option<NonNull<Node<K, V>>> {
        let mut node = self.root;
        let mut res_node = None;
        while let Some(ptr) = node {
            match what.cmp(&ptr.as_ref().key) {
                Ordering::Equal => {
                    res_node = node;
                    break;
                }
                Ordering::Less => {
                    node = ptr.as_ref().left;
                }
                Ordering::Greater => {
                    node = ptr.as_ref().right;
                }
            }
        }
        res_node
    }

    #[inline]
    fn avl_node_link(mut new_node: Box<Node<K, V>>, parent: Option<NonNull<Node<K, V>>>, cmp_node: &mut Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        new_node.parent = parent;
        new_node.height = 0;
        new_node.left = None;
        new_node.right = None;
        *cmp_node = Some(Box::into_raw_non_null(new_node));
        *cmp_node
    }

    unsafe fn avl_left_height(node: NonNull<Node<K, V>>) -> i32 {
        if let Some(node_ptr) = node.as_ref().left {
            node_ptr.as_ref().height
        } else {
            0
        }
    }

    unsafe fn avl_right_height(node: NonNull<Node<K, V>>) -> i32 {
        if let Some(node_ptr) = node.as_ref().right {
            node_ptr.as_ref().height
        } else {
            0
        }
    }

    unsafe fn avl_node_post_insert(&mut self, mut ori_node: Option<NonNull<Node<K, V>>>) {
        ori_node.unwrap().as_mut().height = 1;
        ori_node = ori_node.unwrap().as_ref().parent;
        while let Some(mut node) = ori_node {
            let h0 = Tree::avl_left_height(node);
            let h1 = Tree::avl_right_height(node);
            let height = max(h1, h0) + 1;
            if node.as_ref().height == height {
                break;
            }
            node.as_mut().height = height;
            let diff = h0 - h1;
            if diff <= -2 {
                node = self._avl_node_fix_l(Some(node)).unwrap();
            } else if diff >= 2 {
                node = self._avl_node_fix_r(Some(node)).unwrap();
            }
            ori_node = (*node.as_ptr()).parent;
        }
    }

    unsafe fn _avl_node_fix_l(&mut self, mut node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        let right = node.unwrap().as_ref().right;
        let rh0 = Tree::avl_left_height(right.unwrap());
        let rh1 = Tree::avl_right_height(right.unwrap());
        if rh0 > rh1 {
            let right = self._avl_node_rotate_right(right);
            Tree::_avl_node_height_update(right.unwrap().as_mut().right);
            Tree::_avl_node_height_update(right);
        }
        node = self._avl_node_rotate_left(node);
        Tree::_avl_node_height_update(node.unwrap().as_ref().left);
        Tree::_avl_node_height_update(node);
        node
    }

    unsafe fn _avl_node_fix_r(&mut self, mut node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        let left = node.unwrap().as_ref().left;
        let rh0 = Tree::avl_left_height(left.unwrap());
        let rh1 = Tree::avl_right_height(left.unwrap());
        if rh0 < rh1 {
            let left = self._avl_node_rotate_left(left);
            Tree::_avl_node_height_update(left.unwrap().as_mut().left);
            Tree::_avl_node_height_update(left);
        }
        node = self._avl_node_rotate_right(node);
        Tree::_avl_node_height_update(node.unwrap().as_ref().right);
        Tree::_avl_node_height_update(node);
        node
    }

    unsafe fn _avl_node_height_update(node: Option<NonNull<Node<K, V>>>) {
        let mut node = node.unwrap();
        let h0 = Tree::avl_left_height(node);
        let h1 = Tree::avl_right_height(node);
        node.as_mut().height = max(h0, h1) + 1;
    }

    unsafe fn _avl_node_rotate_right(&mut self, node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        let left = node.unwrap().as_mut().left;
        let parent = node.unwrap().as_mut().parent;
        node.unwrap().as_mut().left = left.unwrap().as_ref().right;
        if let Some(mut x) = left.unwrap().as_ref().right {
            x.as_mut().parent = node;
        }
        left.unwrap().as_mut().right = node;
        left.unwrap().as_mut().parent = parent;
        self._avl_child_replace(node, left, parent);
        node.unwrap().as_mut().parent = left;
        left
    }

    unsafe fn _avl_node_rotate_left(&mut self, node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        let right = node.unwrap().as_mut().right;
        let parent = node.unwrap().as_mut().parent;
        node.unwrap().as_mut().right = right.unwrap().as_ref().left;
        if let Some(mut x) = right.unwrap().as_ref().left {
            x.as_mut().parent = node;
        }
        right.unwrap().as_mut().left = node;
        right.unwrap().as_mut().parent = parent;
        self._avl_child_replace(node, right, parent);
        node.unwrap().as_mut().parent = right;
        right
    }

    unsafe fn _avl_child_replace(&mut self, old_node: Option<NonNull<Node<K, V>>>, new_node: Option<NonNull<Node<K, V>>>, parent: Option<NonNull<Node<K, V>>>) {
        if let Some(mut p) = parent {
            if cmp_node_ptr(&p.as_ref().left, &old_node) {
                p.as_mut().left = new_node;
            } else {
                p.as_mut().right = new_node;
            }
        } else {
            self.root = new_node;
        }
    }

    fn drop_all(&mut self) {
        Tree::drop_node(self.root);
        self.root = None;
    }

    fn drop_node(root: Option<NonNull<Node<K, V>>>) {
        match root {
            Some(ptr) => {
                Tree::drop_node(unsafe { ptr.as_ref() }.left);
                Tree::drop_node(unsafe { ptr.as_ref() }.right);
                unsafe { Box::from_raw(ptr.as_ptr()); }
            }
            None => {}
        }
    }

    fn avl_bst_check(&self) -> bool {
        unsafe {
            let mut node = self.avl_node_first();
            if node.is_none() {
                assert_eq!(self.size(), 0);
                return true;
            }
            let mut cnt = 1usize;
            let mut value = &((*node.unwrap().as_ptr()).key);
            node = Node::node_next(node);
            while node.is_some() {
                let x = &((*node.unwrap().as_ptr()).key);
                if x <= value {
                    return false;
                }
                value = x;
                node = Node::node_next(node);
                cnt += 1;
            }
            assert_eq!(cnt, self.count);
            return true;
        }
    }

    fn avl_bst_check_reverse(&self) -> bool {
        unsafe {
            let mut node = self.avl_node_last();
            if node.is_none() {
                assert_eq!(self.size(), 0);
                return true;
            }
            let mut cnt = 1usize;
            let mut value = &((*node.unwrap().as_ptr()).key);
            node = Node::node_prev(node);
            while node.is_some() {
                let x = &((*node.unwrap().as_ptr()).key);
                if x >= value {
                    return false;
                }
                value = x;
                node = Node::node_prev(node);
                cnt += 1;
            }
            assert_eq!(cnt, self.count);
            return true;
        }
    }

    unsafe fn avl_tree_remove(&mut self, node: Option<NonNull<Node<K, V>>>) -> Option<NonNull<Node<K, V>>> {
        if node.is_none() {
            return None;
        }
        self.avl_node_erase(node);
        self.count -= 1;
        node
    }

    fn avl_tree_pop(&mut self, what: &K) -> Option<V> {
        unsafe {
            let node = self.avl_node_find(what);
            match self.avl_tree_remove(node) {
                None => None,
                Some(x) => {
                    let ptr = x.as_ptr();
                    let res = (*ptr).value.clone();
                    Box::from_raw(ptr);
                    Some(res)
                }
            }
        }
    }

    unsafe fn avl_node_erase(&mut self, mut node: Option<NonNull<Node<K, V>>>) {
        let mut parent = None;
        let ptr = node.unwrap();
        if ptr.as_ref().left.is_some() && ptr.as_ref().right.is_some() {
            let old = node;
            node = node.unwrap().as_ref().right;
            while let Some(left) = node.unwrap().as_ref().left {
                node = Some(left);
            }
            let child = node.unwrap().as_ref().right;
            parent = node.unwrap().as_ref().parent;
            if let Some(mut x) = child {
                x.as_mut().parent = parent;
            }
            self._avl_child_replace(node, child, parent);
            if cmp_node_ptr(&node.unwrap().as_ref().parent, &old) {
                parent = node;
            }
            {
                let new_ptr = node.as_mut().unwrap().as_mut();
                let old_ptr = old.as_ref().unwrap().as_ref();
                new_ptr.left = old_ptr.left;
                new_ptr.right = old_ptr.right;
                new_ptr.parent = old_ptr.parent;
                new_ptr.height = old_ptr.height;
            }
            self._avl_child_replace(old, node, old.unwrap().as_ref().parent);
            old.unwrap().as_ref().left.unwrap().as_mut().parent = node;
            if let Some(ref mut x) = old.unwrap().as_mut().right {
                x.as_mut().parent = node;
            }
        } else {
            let node_ptr = node.unwrap();
            let child = if node_ptr.as_ref().left.is_none() {
                node_ptr.as_ref().right
            } else {
                node_ptr.as_ref().left
            };
            parent = node_ptr.as_ref().parent;
            self._avl_child_replace(node, child, parent);
            if let Some(mut x) = child {
                x.as_mut().parent = parent;
            }
        }
        if parent.is_some() {
            self._avl_node_rebalance(parent);
        }
    }

    #[inline]
    unsafe fn _avl_node_rebalance(&mut self, mut node: Option<NonNull<Node<K, V>>>) {
        while node.is_some() {
            let mut ptr = node.unwrap();
            let h0 = Tree::avl_left_height(ptr);
            let h1 = Tree::avl_right_height(ptr);
            let diff = h0 - h1;
            let height = max(h0, h1) + 1;
            if ptr.as_ref().height != height {
                ptr.as_mut().height = height;
            } else if diff >= -1 && diff <= 1 {
                break;
            }
            if diff <= -2 {
                node = self._avl_node_fix_l(node);
            } else if diff >= 2 {
                node = self._avl_node_fix_r(node);
            }
            node = node.unwrap().as_ref().parent;
        }
    }
}

#[test]
fn just_for_compile() {}

impl<K, V> Drop for Tree<K, V> where K: Ord, V: Clone {
    fn drop(&mut self) {
        self.drop_all();
    }
}

mod test {
    extern crate rand;

    use avl::Tree;
    use std::cmp::Ordering;

    type DefaultType = Tree<i32, Option<i32>>;

    #[test]
    fn test_avl_erase() {
        let test_num = 1000usize;
        let mut t = default_build_avl(test_num);
        for _ in 0..200 {
            let x = (rand::random::<usize>() % test_num) as i32;
            unsafe {
                match t.avl_tree_pop(&x) {
                    None => {},
                    Some(res) => {
                        assert_eq!(res.unwrap(), -x);
                    }
                }
                assert!(t.avl_node_find(&x).is_none());
            }
        }
        assert!(t.avl_bst_check());
        assert!(t.avl_bst_check_reverse());
    }

    #[test]
    fn test_avl_basic() {
        let mut t = DefaultType::new();
        unsafe {
            assert!(t.root.is_none());
            t.avl_add_element(3, None);
            assert_eq!(t.root.unwrap().as_ref().key, 3);
            assert_eq!(t.root.unwrap().as_ref().height, 1);
            assert!(t.root.unwrap().as_ref().left.is_none());
            assert!(t.root.unwrap().as_ref().right.is_none());

            t.avl_add_element(2, None);
            assert_eq!(t.root.unwrap().as_ref().key, 3);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
            assert!(t.root.unwrap().as_ref().left.is_some());
            assert!(t.root.unwrap().as_ref().right.is_none());
            assert_eq!(t.root.unwrap().as_ref().left.unwrap().as_ref().parent.unwrap().as_ptr(), t.root.unwrap().as_ptr());
        }
    }


    #[test]
    fn test_avl_rotate_right() {
        let mut t = DefaultType::new();
        unsafe {
            t.avl_add_element(3, None);
            assert_eq!(t.root.unwrap().as_ref().key, 3);
            assert_eq!(t.root.unwrap().as_ref().height, 1);
            t.avl_add_element(2, None);
            assert_eq!(t.root.unwrap().as_ref().key, 3);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
            t.avl_add_element(1, None);
            assert_eq!(t.root.unwrap().as_ref().key, 2);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
        }
    }

    #[test]
    fn test_avl_rotate_left() {
        let mut t = DefaultType::new();
        unsafe {
            t.avl_add_element(1, None);
            assert_eq!(t.root.unwrap().as_ref().key, 1);
            assert_eq!(t.root.unwrap().as_ref().height, 1);
            t.avl_add_element(2, None);
            assert_eq!(t.root.unwrap().as_ref().key, 1);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
            t.avl_add_element(3, None);
            assert_eq!(t.root.unwrap().as_ref().key, 2);
            assert_eq!(t.root.unwrap().as_ref().height, 2);
        }
    }

    #[test]
    fn test_avl_element_cmp() {
        #[derive(Eq)]
        struct MyData {
            a: i32,
        }

        impl Ord for MyData {
            fn cmp(&self, other: &Self) -> Ordering {
                self.a.cmp(&other.a)
            }
        }

        impl PartialOrd for MyData {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl PartialEq for MyData {
            fn eq(&self, other: &Self) -> bool {
                self.a == other.a
            }
        }

        let mut t = Tree::<MyData, Option<i32>>::new();
        unsafe {
            t.avl_add_element(MyData { a: 1 }, None);
            assert!(t.root.unwrap().as_ref().key == MyData { a: 1 });
            assert_eq!(t.root.unwrap().as_ref().height, 1);
            t.avl_add_element(MyData { a: 2 }, None);
            assert!(t.root.unwrap().as_ref().key == MyData { a: 1 });
            assert_eq!(t.root.unwrap().as_ref().height, 2);
        }
    }

    #[test]
    fn test_avl_duplicate() {
        let mut t = DefaultType::new();
        let mut is_duplicate = t.avl_add_element(1, None);
        assert!(!is_duplicate);
        assert!(unsafe { t.avl_node_find(&1) }.is_some());
        is_duplicate = t.avl_add_element(1, None);
        assert!(is_duplicate);
    }

    #[test]
    fn test_avl_find() {
        let mut t = default_build_avl(10);
        assert!(unsafe { t.avl_node_find(&11) }.is_none());
        t.avl_add_element(11, Some(2333333));
        unsafe {
            assert_eq!(t.avl_node_find(&11).unwrap().as_ref().value.unwrap(), 2333333);
        }
    }

    fn default_build_avl(n: usize) -> DefaultType {
        let mut v = vec![0i32; n];
        for idx in 0..v.len() {
            v[idx] = idx as i32;
            let pos = rand::random::<usize>() % (idx + 1);
            assert!(pos <= idx);
            v.swap(idx, pos);
        }
        let mut t = DefaultType::new();
        assert_eq!(t.size(), 0);
        for d in &v {
            t.avl_add_element(*d, Some(-*d));
        }
        t
    }

    #[test]
    fn test_avl_validate() {
        unsafe {
            let test_num = 1000usize;
            let t = default_build_avl(test_num);
            assert_eq!(t.size(), test_num);
            assert_eq!(t.root.unwrap().as_ref().height, 12);
            let left = t.root.unwrap().as_ref().left;
            assert!(left.unwrap().as_ref().height <= 11);
            assert!(left.unwrap().as_ref().height >= 10);
            let right = t.root.unwrap().as_ref().right;
            assert!(right.unwrap().as_ref().height <= 11);
            assert!(right.unwrap().as_ref().height >= 10);

            assert!(t.avl_bst_check());
            assert!(t.avl_bst_check_reverse());
        }
    }
}
