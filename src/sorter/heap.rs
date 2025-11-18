// A non-growable array-backed binary heap. Assumed to have at least one element.
pub struct Heap<'a> {
    repr: &'a mut [usize],
    n: usize,
}

impl<'a> Heap<'a> {
    pub fn new(repr: &'a mut [usize]) -> Self {
        let n = repr.len();
        let mut _self = Self { repr, n };
        _self
    }

    pub fn swap(&mut self, a: Node, b: Node) {
        self.repr.swap(a.index, b.index);
    }

    // Abandon n trailing nodes.
    pub fn abandon(&mut self, nodes: usize) {
        self.n -= nodes;
    }

    pub fn root(&self) -> Node {
        assert!(self.nodes() >= 1);
        Node::of(0, self)
    }

    pub fn node(&self, index: usize) -> Node {
        Node::of(index, self)
    }

    pub fn nodes(&self) -> usize {
        self.n
    }

    pub fn repr(&'a self) -> &'a [usize] {
        self.repr
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Node {
    index: usize,
}

impl Node {
    pub fn of(index: usize, heap: &Heap<'_>) -> Self {
        assert!(index < heap.nodes());
        Self { index }
    }

    pub fn value(&self, heap: &Heap<'_>) -> usize {
        heap.repr[self.index]
    }

    pub fn children(&self, heap: &Heap<'_>) -> [Option<Self>; 2] {
        let left = (2 * self.index) + 1;
        let right = (2 * self.index) + 2;

        [
            (left < heap.nodes()).then(|| Node::of(left, heap)),
            (right < heap.nodes()).then(|| Node::of(right, heap)),
        ]
    }
}
