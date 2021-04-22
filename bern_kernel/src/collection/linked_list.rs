#![allow(unused)]

use core::ptr;
use core::ptr::NonNull;
use core::mem::MaybeUninit;
use core::cell::RefCell;
use core::borrow::BorrowMut;
use super::boxed::Box;

/// The goal here is to create a fast and efficient linked list
/// Lists use an array of nodes as memory pool, the array must be static.
/// In contrast to `std::collections::LinkedList` you can never copy the inner
/// value out of the list.

type Link<T> = Option<NonNull<Node<T>>>;

/******************************************************************************/

// Copy needed for initialization
#[derive(Debug, Copy, Clone)]
pub struct Node<T> {
    inner: T,
    prev: Link<T>,
    next: Link<T>,
}

impl<T> Node<T> {
    pub fn new(element: T) -> Self {
        Node {
            inner: element,
            prev: None,
            next: None,
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

/******************************************************************************/

#[derive(Debug)]
pub struct LinkedList<T,P>
    where P: ListAllocator<T> + 'static
{
    head: Link<T>,
    tail: Link<T>,
    pool: &'static P,
    len: usize,
}

/// base on std::collections::LinkedList and https://rust-unofficial.github.io/too-many-lists
impl<T,P> LinkedList<T,P>
    where P: ListAllocator<T> + 'static
{
    pub fn new(pool: &'static P) -> Self {
        LinkedList {
            head: None,
            tail: None,
            pool,
            len: 0,
        }
    }

    pub fn insert_back(&mut self, element: T) -> Result<(), Error> {
        let node = self.pool.insert(Node::new(element));
        node.map(|n| {
            self.push_back(n);
        })
    }

    pub fn push_back(&mut self, mut node: Box<Node<T>>) {
        node.prev = self.tail;

        let link = node.into_nonnull();
        // NOTE(unsafe):  we check tail is Some()
        unsafe {
            match self.tail {
                None => self.head = link,
                Some(mut tail) => tail.as_mut().next = link,
            }
        }

        self.tail = link;
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<Box<Node<T>>> {
        let mut front = self.head.take();

        match front {
            Some(mut node) => unsafe {
                self.head = node.as_ref().next;
                if let Some(mut head) = self.head {
                    head.as_mut().prev = None;
                }
                if self.tail == Some(node) {
                    self.tail = node.as_ref().next;
                }
                node.as_mut().next = None;
                self.len -= 1;
                Some(Box::from_raw(node))
            },
            None => None,
        }
    }

    pub fn front(&self) -> Option<&T> {
        self.head.map(|front| unsafe { front.as_ref().inner() })
    }

    pub fn back(&self) -> Option<&T> {
        self.tail.map(|back| unsafe { back.as_ref().inner() })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// a node is only allowed to be unliked once
    unsafe fn unlink(&mut self, node: &mut Node<T>) -> Box<Node<T>> {
        match node.prev {
            Some(mut prev) => prev.as_mut().next = node.next,
            None => self.head = node.next,
        };

        match node.next {
            Some(mut next) => next.as_mut().prev = node.prev,
            None => self.tail = node.prev,
        };

        node.prev = None;
        node.next = None;
        self.len -= 1;

        Box::from_raw(NonNull::new_unchecked(node))
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.map(|node| unsafe { & *node.as_ptr() }),
        }
    }

    pub fn iter_mut(&self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.map(|node| unsafe { &mut *node.as_ptr() })
        }
    }

    pub fn cursor_front_mut(&mut self) -> Cursor<'_, T, P> {
        Cursor { node: self.head, list: self }
    }
}

/******************************************************************************/

pub struct Iter<'a, T>
{
    next: Option<&'a Node<T>>,
}

impl<'a,T> Iterator for Iter<'a,T>
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| unsafe {
            self.next = node.next.map(|next| next.as_ref());
            node.inner()
        })
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| unsafe {
            self.next = node.next.map(|mut next| next.as_mut());
            node.inner_mut()
        })
    }
}

/******************************************************************************/

#[derive(Debug)]
pub struct Cursor<'a,T,P>
    where P: ListAllocator<T> + 'static
{
    node: Link<T>,
    list: &'a mut LinkedList<T,P>,
}

impl<'a, T, P> Cursor<'a, T, P>
    where P: ListAllocator<T> + Sized
{
    pub fn inner(&self) -> Option<&T> {
        self.node.map(|node| unsafe { node.as_ref() }.inner())
    }

    pub fn inner_mut(&self) -> Option<&mut T> {
        self.node.map(|mut node| unsafe { node.as_mut() }.inner_mut())
    }

    pub fn move_next(&mut self) {
        if let Some(node) = self.node {
            self.node = unsafe { node.as_ref().next };
        }
    }

    pub fn take(&mut self) -> Option<Box<Node<T>>> {
        self.node.map(|mut node|
            unsafe {
                self.node = node.as_ref().next;
                self.list.unlink(node.as_mut())
            })
    }
}
/******************************************************************************/

#[derive(Debug, PartialEq)]
pub enum Error {
    OutOfMemory,
    Unknown,
}

pub trait ListAllocator<T> {
    fn insert(&self, node: Node<T>) -> Result<Box<Node<T>>, Error>;
    // todo: drop
}

#[derive(Debug)]
pub struct StaticListPool<T, const N: usize> {
    pool: RefCell<[Option<Node<T>>; N]>,
}

impl<T, const N: usize> StaticListPool<T, {N}>
{
    pub const fn new(array: [Option<Node<T>>; N]) -> Self {
        let mut pool = RefCell::new(array);
        StaticListPool {
            pool,
        }
    }
}

// todo: make sync safe!
unsafe impl<T, const N: usize> Sync for StaticListPool<T, {N}> {}


impl<T, const N: usize> ListAllocator<T> for StaticListPool<T, {N}> {
    fn insert(&self, node: Node<T>) -> Result<Box<Node<T>>, Error> {
        for item in self.pool.borrow_mut().iter_mut() {
            if item.is_none() {
                *item = Some(node);
                match item {
                    Some(n) => unsafe {
                        return Ok(Box::from_raw(NonNull::new_unchecked(n as *mut _)))
                    },
                    _ => return Err(Error::Unknown),
                }
            }
        }
        return Err(Error::OutOfMemory);
    }
}

/******************************************************************************/

#[cfg(all(test, not(target_arch = "arm")))]
mod tests {
    use super::*;
    use core::borrow::Borrow;

    #[derive(Debug, Copy, Clone)]
    struct MyStruct {
        pub id: u32,
    }

    #[test]
    fn one_node() {
        static POOL: StaticListPool<MyStruct,16> = StaticListPool::new([None; 16]);
        let node_0 = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
        assert_eq!(node_0.prev, None);
        assert_eq!(node_0.next, None);

        let mut list = LinkedList::new(&POOL);
        assert_eq!(list.head, None);
        assert_eq!(list.tail, None);

        list.push_back(node_0);
        assert_ne!(list.head, None);
        assert_eq!(list.tail, list.head);
        unsafe {
            assert_eq!(list.head.unwrap().as_ref().prev, None);
            assert_eq!(list.head.unwrap().as_ref().next, None);
        }

        let node = list.pop_front();

        assert_eq!(list.head, None);
        assert_eq!(list.tail, None);
        assert_eq!(node.as_ref().unwrap().prev, None);
        assert_eq!(node.as_ref().unwrap().next, None);
    }

    #[test]
    fn length() {
        static POOL: StaticListPool<MyStruct,16> = StaticListPool::new([None; 16]);

        let mut list = LinkedList::new(&POOL);
        assert_eq!(list.len(), 0);
        list.pop_front();
        assert_eq!(list.len(), 0);
        list.insert_back(MyStruct { id: 42 });
        assert_eq!(list.len(), 1);
        list.pop_front();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn pushing_and_popping() {
        static POOL: StaticListPool<MyStruct,16> = StaticListPool::new([None; 16]);

        let mut list = LinkedList::new(&POOL);
        list.insert_back(MyStruct { id: 42 });
        list.insert_back(MyStruct { id: 43 });

        let mut another_list = LinkedList::new(&POOL);
        list.insert_back(MyStruct { id: 44 });

        let mut front = list.pop_front();
        assert_eq!(front.as_mut().unwrap().inner().id, 42);
        another_list.push_back(front.unwrap());

        assert_eq!(another_list.back().unwrap().id, 42);
    }

    #[test]
    fn pool_overflow() {
        static POOL: StaticListPool<MyStruct,16> = StaticListPool::new([None; 16]);

        let mut list = LinkedList::new(&POOL);
        for i in 0..16 {
            assert_eq!(list.insert_back(MyStruct { id: i }), Ok(()));
        }
        assert_eq!(list.insert_back(MyStruct { id: 16 }), Err(Error::OutOfMemory));
    }

    #[test]
    fn iterate() {
        static POOL: StaticListPool<MyStruct,16> = StaticListPool::new([None; 16]);
        let node_0 = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
        let node_1 = POOL.insert(Node::new(MyStruct { id: 43 })).unwrap();
        let node_2 = POOL.insert(Node::new(MyStruct { id: 44 })).unwrap();

        let mut list = LinkedList::new(&POOL);
        list.push_back(node_0);
        list.push_back(node_1);
        list.push_back(node_2);

        let truth = vec![42,43,44,45];
        for (i, element) in list.iter().enumerate() {
            assert_eq!(element.id, truth[i]);
        }
        // everything should still work fine
        for (i, element) in list.iter().enumerate() {
            assert_eq!(element.id, truth[i]);
        }
    }

    #[test]
    fn iterate_mut() {
        static POOL: StaticListPool<MyStruct,16> = StaticListPool::new([None; 16]);
        let node_0 = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
        let node_1 = POOL.insert(Node::new(MyStruct { id: 43 })).unwrap();
        let node_2 = POOL.insert(Node::new(MyStruct { id: 44 })).unwrap();

        let mut list = LinkedList::new(&POOL);
        list.push_back(node_0);
        list.push_back(node_1);
        list.push_back(node_2);

        let truth = vec![42,43,44,45];
        for (i, element) in list.iter_mut().enumerate() {
            assert_eq!(element.id, truth[i]);
            element.id = i as u32;
        }
        // values should have changed
        let truth = vec![0,1,2,3];
        for (i, element) in list.iter().enumerate() {
            assert_eq!(element.id, truth[i]);
        }
    }

    #[test]
    fn find_and_take() {
        static POOL: StaticListPool<MyStruct,16> = StaticListPool::new([None; 16]);
        let node_0 = POOL.insert(Node::new(MyStruct { id: 42 })).unwrap();
        let node_1 = POOL.insert(Node::new(MyStruct { id: 43 })).unwrap();
        let node_2 = POOL.insert(Node::new(MyStruct { id: 44 })).unwrap();

        let mut list = LinkedList::new(&POOL);
        list.push_back(node_0);
        list.push_back(node_1);
        list.push_back(node_2);

        let mut another_list = LinkedList::new(&POOL);

        let mut cursor = list.cursor_front_mut();
        let mut target: Option<Box<Node<MyStruct>>> = None;
        while let Some(element) = cursor.inner() {
            if element.id == 43 {
                target = cursor.take();
                break;
            }
            cursor.move_next();
        }
        another_list.push_back(target.unwrap());

        let truth = vec![42,44];
        for (i, element) in list.iter().enumerate() {
            assert_eq!(element.id, truth[i]);
        }

        for element in another_list.iter() {
            assert_eq!(element.id, 43);
        }
    }
}