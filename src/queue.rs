use crate::{check_concern, hostcalls, RootContext, Status};

/// Shared Queues in proxy-wasm are a FIFO MPMC queue with *no message duplication*.
/// Any WASM VM can resolve a queue or register new ones in their own VM ID.
/// Any WASM VM can dequeue data, which will globally dequeue that item. Messages are not replicated to each WASM VM.
/// When broadcasting data to many WASM VMs, it's advised to have a scheme where each thread can register it's own inbound queue, then enqueue the name of said queue to the centralized source of data. That source then enqueues to each WASM VM's queue individually.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Queue(pub(crate) u32);

impl Queue {
    /// Registers a new queue under a given name. Names are globally unique underneath a single VM ID.
    /// Re-registering the same name from *any WASM VM* in the same VM ID will overwrite the previous registration of that name, and is not advised.
    pub fn register(name: impl AsRef<str>) -> Result<Self, Status> {
        hostcalls::register_shared_queue(name.as_ref()).map(Self)
    }

    /// Resolves an existing queue for a given name in the given VM ID.
    pub fn resolve(vm_id: impl AsRef<str>, name: impl AsRef<str>) -> Result<Option<Self>, Status> {
        hostcalls::resolve_shared_queue(vm_id.as_ref(), name.as_ref()).map(|x| x.map(Self))
    }

    /// Remove an item from this queue, if any is present. Returns `Ok(None)` when no data is enqueued.
    /// Note that this is not VM-local and any message can only be received by one dequeue operation *anywhere*.
    pub fn dequeue(&self) -> Result<Option<Vec<u8>>, Status> {
        hostcalls::dequeue_shared_queue(self.0)
    }

    /// Enqueues a new item into this queue.
    pub fn enqueue(&self, value: impl AsRef<[u8]>) -> Result<(), Status> {
        hostcalls::enqueue_shared_queue(self.0, value)
    }

    /// Registers a callback that is called whenever data is available in the queue to be dequeued.
    /// Only one of `on_enqueue` or `on_receive` can be set at the same time.
    pub fn on_enqueue<R: RootContext>(self, callback: impl FnMut(&mut R, Queue) + 'static) -> Self {
        crate::dispatcher::register_queue_callback(self.0, callback);
        self
    }

    /// Registers a callback that is called whenever data is available in the queue to be dequeued.
    /// Also dequeues anything on the queue. It may call the callback multiple times for each item, if multiple are present.
    /// Only one of `on_enqueue` or `on_receive` can be set at the same time.
    pub fn on_receive<R: RootContext>(
        self,
        mut callback: impl FnMut(&mut R, Queue, Vec<u8>) + 'static,
    ) -> Self {
        crate::dispatcher::register_queue_callback(self.0, move |root, queue| {
            while let Some(dequeued) = check_concern("queue-receive", queue.dequeue()).flatten() {
                callback(root, queue, dequeued);
            }
        });
        self
    }
}

impl PartialEq<u32> for Queue {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Queue> for u32 {
    fn eq(&self, other: &Queue) -> bool {
        other == self
    }
}
