use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use doom_fish_utils::panic_safe::catch_user_panic_result;
use doom_fish_utils::stream::AsyncStreamSender;

pub struct HandlerState<H: ?Sized, E> {
    handler: Mutex<Box<H>>,
    pub sender: AsyncStreamSender<E>,
}

impl<H: ?Sized, E> HandlerState<H, E> {
    pub fn new(handler: Box<H>, sender: AsyncStreamSender<E>) -> Self {
        Self {
            handler: Mutex::new(handler),
            sender,
        }
    }

    pub fn call<R>(&self, site: &str, f: impl FnOnce(&mut H) -> R) -> Option<R> {
        let mut handler = self.handler.lock().unwrap_or_else(PoisonError::into_inner);
        catch_user_panic_result(site, || f(&mut **handler))
    }
}

type Slot<H, E> = Option<(u64, Arc<HandlerState<H, E>>)>;

pub struct Registry<H: ?Sized, E> {
    slot: Mutex<Slot<H, E>>,
}

static NEXT_REGISTRATION_ID: AtomicU64 = AtomicU64::new(1);

impl<H: ?Sized, E> Registry<H, E> {
    pub const fn new() -> Self {
        Self {
            slot: Mutex::new(None),
        }
    }

    pub fn install(&self, state: HandlerState<H, E>) -> u64 {
        let id = NEXT_REGISTRATION_ID.fetch_add(1, Ordering::Relaxed);
        let previous = self.lock().replace((id, Arc::new(state)));
        drop(previous);
        id
    }

    pub fn current(&self) -> Option<Arc<HandlerState<H, E>>> {
        self.lock().as_ref().map(|(_, state)| Arc::clone(state))
    }

    pub fn remove(&self, id: u64) {
        let removed = {
            let mut slot = self.lock();
            if slot.as_ref().is_some_and(|(current, _)| *current == id) {
                slot.take()
            } else {
                None
            }
        };
        drop(removed);
    }

    fn lock(&self) -> MutexGuard<'_, Slot<H, E>> {
        self.slot.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use doom_fish_utils::stream::BoundedAsyncStream;

    use super::{HandlerState, Registry};

    trait Counter: Send {
        fn hit(&mut self) -> usize;
    }

    struct Probe {
        hits: usize,
        drops: Arc<AtomicUsize>,
    }

    impl Counter for Probe {
        fn hit(&mut self) -> usize {
            self.hits += 1;
            self.hits
        }
    }

    impl Drop for Probe {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn state(
        drops: &Arc<AtomicUsize>,
    ) -> (BoundedAsyncStream<u32>, HandlerState<dyn Counter, u32>) {
        let (stream, sender) = BoundedAsyncStream::new(4);
        let handler: Box<dyn Counter> = Box::new(Probe {
            hits: 0,
            drops: Arc::clone(drops),
        });
        (stream, HandlerState::new(handler, sender))
    }

    #[test]
    fn call_contains_panics_and_keeps_the_handler_usable() {
        let drops = Arc::new(AtomicUsize::new(0));
        let (_stream, state) = state(&drops);

        assert_eq!(state.call("hit", Counter::hit), Some(1));
        assert_eq!(
            state.call("panic", |_| -> usize { panic!("handler panic") }),
            None
        );
        assert_eq!(state.call("hit", Counter::hit), Some(2));
    }

    #[test]
    fn handler_can_reach_the_registry_while_it_runs() {
        static REGISTRY: Registry<dyn Counter, u32> = Registry::new();
        let drops = Arc::new(AtomicUsize::new(0));
        let (_first_stream, first) = state(&drops);
        let first_id = REGISTRY.install(first);

        let current = REGISTRY.current().unwrap();
        let replaced = current.call("reentrant install", |handler| {
            let (_second_stream, second) = state(&drops);
            let second_id = REGISTRY.install(second);
            REGISTRY.remove(first_id);
            handler.hit();
            second_id
        });
        let second_id = replaced.unwrap();
        assert_ne!(first_id, second_id);
        assert_eq!(drops.load(Ordering::SeqCst), 0);

        drop(current);
        assert_eq!(drops.load(Ordering::SeqCst), 1);

        REGISTRY.remove(first_id);
        assert!(REGISTRY.current().is_some());
        REGISTRY.remove(second_id);
        assert!(REGISTRY.current().is_none());
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn replacing_a_registration_closes_the_previous_stream() {
        static REGISTRY: Registry<dyn Counter, u32> = Registry::new();
        let drops = Arc::new(AtomicUsize::new(0));
        let (first_stream, first) = state(&drops);
        let first_id = REGISTRY.install(first);
        let (second_stream, second) = state(&drops);
        let second_id = REGISTRY.install(second);

        assert!(first_stream.is_closed());
        assert!(!second_stream.is_closed());
        assert_eq!(drops.load(Ordering::SeqCst), 1);

        REGISTRY.remove(first_id);
        assert!(!second_stream.is_closed());
        REGISTRY.remove(second_id);
        assert!(second_stream.is_closed());
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
}
