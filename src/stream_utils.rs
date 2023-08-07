use futures::stream::Stream;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

pub struct StreamWithClosure<S>
where
    S: Stream,
{
    inner: S,
    on_end: Option<Arc<Mutex<Option<Box<dyn FnOnce() + Send + 'static>>>>>,
}

impl<S> StreamWithClosure<S>
where
    S: Stream,
{
    pub fn new(inner: S, on_end: Box<dyn FnOnce() + Send + 'static>) -> Self {
        StreamWithClosure {
            inner,
            on_end: Some(Arc::new(Mutex::new(Some(on_end)))),
        }
    }
}

impl<S> Stream for StreamWithClosure<S>
where
    S: Stream + Unpin,
{
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let next = Pin::new(&mut self.inner).poll_next(cx);

        let next = match next {
            Poll::Ready(n) => n,
            Poll::Pending => return Poll::Pending,
        };

        if next.is_some() {
            // The inner stream produced a value.
            return Poll::Ready(next);
        }

        // The inner stream has ended.
        if let Some(on_end) = self.on_end.take() {
            let closure = Arc::clone(&on_end);
            tokio::spawn(async move {
                let mut on_end = closure.lock().unwrap();
                if let Some(closure) = on_end.take() {
                    closure();
                }
            });
        }

        Poll::Ready(None)
    }
}
