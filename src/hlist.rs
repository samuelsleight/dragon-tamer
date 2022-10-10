pub struct Cons<Value, Tail: HList> {
    value: Value,
    tail: Tail,
}

pub trait HList: Sized {
    fn cons<Value>(self, value: Value) -> Cons<Value, Self> {
        Cons { value, tail: self }
    }
}

pub trait IndexableBy<const Index: usize>: HList {
    type Result;

    fn index(&self) -> &Self::Result;
}

impl<Value> IndexableBy<0> for Cons<Value, ()> {
    type Result = Value;

    fn index(&self) -> &Self::Result {
        &self.value
    }
}

impl<Value, A> IndexableBy<1> for Cons<Value, Cons<A, ()>> {
    type Result = Value;

    fn index(&self) -> &Self::Result {
        &self.value
    }
}

impl<Value, A, B> IndexableBy<2> for Cons<Value, Cons<A, Cons<B, ()>>> {
    type Result = Value;

    fn index(&self) -> &Self::Result {
        &self.value
    }
}

impl<Value, A, B, C> IndexableBy<3> for Cons<Value, Cons<A, Cons<B, Cons<C, ()>>>> {
    type Result = Value;

    fn index(&self) -> &Self::Result {
        &self.value
    }
}

impl<const Index: usize, Value, Tail: IndexableBy<{ Index }>> IndexableBy<{ Index }>
    for Cons<Value, Tail>
{
    type Result = <Tail as IndexableBy<{ Index }>>::Result;

    fn index(&self) -> &Self::Result {
        self.tail.index()
    }
}

impl HList for () {}

impl<Value, Tail: HList> HList for Cons<Value, Tail> {}

impl<Value> Cons<Value, ()> {
    pub const fn new(value: Value) -> Self {
        Self { value, tail: () }
    }
}

impl<Value, Tail: HList> Cons<Value, Tail> {
    pub fn cons<Head>(self, value: Head) -> Cons<Head, Self> {
        Cons { value, tail: self }
    }

    pub fn get<const Index: usize>(&self) -> &<Self as IndexableBy<Index>>::Result
    where
        Self: IndexableBy<Index>,
    {
        self.index()
    }
}
