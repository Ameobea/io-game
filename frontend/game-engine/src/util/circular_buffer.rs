#[derive(Debug)]
pub struct CircularBuffer<T: Default> {
    pub front: usize,
    pub back: usize,
    pub data: Vec<T>,
    pub full: bool,
}

pub struct CircularBufferIter<'a, T: Default + 'a> {
    cur: usize,
    full: bool,
    inner: &'a CircularBuffer<T>,
}

impl<'a, T: Default + 'a> CircularBufferIter<'a, T> {
    pub fn new(inner: &'a CircularBuffer<T>) -> Self {
        CircularBufferIter {
            cur: inner.back,
            full: inner.full,
            inner,
        }
    }
}

impl<'a, T: Default + 'a> Iterator for CircularBufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.inner.front {
            if self.full {
                self.full = false;
            } else {
                return None;
            }
        }

        let item = &self.inner.data[self.cur];
        self.cur = self.inner.next_index(self.cur);
        Some(item)
    }
}

impl<T: Default> CircularBuffer<T> {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        let mut data = Vec::with_capacity(size);
        data.resize_default(size);

        CircularBuffer {
            front: 0,
            back: 0,
            data,
            full: false,
        }
    }

    #[inline]
    fn size(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn next_index(&self, cur_index: usize) -> usize {
        if cur_index < self.size() - 1 {
            cur_index + 1
        } else {
            0
        }
    }

    pub fn push(&mut self, item: T) {
        if self.front == self.back && self.full {
            self.back = self.next_index(self.back);
        }

        self.data[self.front] = item;
        self.front = self.next_index(self.front);

        if self.front == self.back {
            self.full = true
        }
    }

    pub fn pop<'a>(&'a mut self) -> Option<&'a T> {
        if self.front == self.back {
            None
        } else {
            let item = &self.data[self.back];
            self.back = self.next_index(self.back);
            self.full = false;
            Some(item)
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        CircularBufferIter::new(self)
    }
}

impl<T: Clone + Default> CircularBuffer<T> {
    pub fn pop_clone(&mut self) -> Option<T> {
        if self.front == self.back {
            None
        } else {
            let item = self.data[self.back].clone();
            self.back = self.next_index(self.back);
            Some(item)
        }
    }
}

#[test]
fn range_push_pop() {
    let mut q: CircularBuffer<u8> = CircularBuffer::new(4);
    assert_eq!(q.pop(), None);
    q.push(10);
    q.push(11);
    assert_eq!(q.pop_clone(), Some(10));
    assert_eq!(q.pop_clone(), Some(11));
    assert_eq!(q.pop(), None);
}

#[test]
fn range_iter() {
    let mut q: CircularBuffer<u8> = CircularBuffer::new(10);
    let empty: Vec<u8> = vec![];
    assert_eq!(q.iter().cloned().collect::<Vec<_>>(), empty);
    q.push(10);
    q.push(11);
    assert_eq!(q.iter().cloned().collect::<Vec<_>>(), &[10, 11]);
    q.pop();
    q.pop();
    assert_eq!(q.iter().cloned().collect::<Vec<_>>(), empty);
}

#[test]
fn iter_rollover() {
    let mut q: CircularBuffer<u8> = CircularBuffer::new(3);
    q.push(1);
    q.push(2);
    q.push(3);
    println!("{:?}", q);
    assert_eq!(q.iter().cloned().collect::<Vec<_>>(), vec![1, 2, 3]);
    q.push(4);
    println!("{:?}", q);
    assert_eq!(q.iter().cloned().collect::<Vec<_>>(), vec![2, 3, 4]);
}
