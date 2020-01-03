pub struct PeekWhile<'a, I, F>
where
    I: Iterator + 'a,
{
    iter: &'a mut std::iter::Peekable<I>,
    f: F,
}

impl<'a, I, F> Iterator for PeekWhile<'a, I, F>
where
    I: Iterator + 'a,
    F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
{
    type Item = <I as Iterator>::Item;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let &mut PeekWhile {
            ref mut iter,
            ref mut f,
        } = self;
        if iter.peek().map(f).unwrap_or(false) {
            iter.next()
        } else {
            None
        }
    }
}

pub fn peek_while<'a, I, F>(iter: &'a mut std::iter::Peekable<I>, f: F) -> PeekWhile<'a, I, F>
where
    I: Iterator + 'a,
    F: for<'b> FnMut(&'b <I as Iterator>::Item) -> bool,
{
    PeekWhile { iter, f }
}
