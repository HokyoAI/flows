use super::Reset;
use heapless::mpmc::MpMcQueue;

pub struct DataChannel<UD: 'static, FD: 'static, const N: usize> {
    pub user_data: MpMcQueue<UD, N>,
    pub fn_data: MpMcQueue<FD, N>,
}

impl<UD: 'static, FD: 'static, const N: usize> Default for DataChannel<UD, FD, N> {
    fn default() -> Self {
        DataChannel {
            user_data: MpMcQueue::new(),
            fn_data: MpMcQueue::new(),
        }
    }
}

impl<UD: 'static, FD: 'static, const N: usize> Reset for DataChannel<UD, FD, N> {
    fn reset(&self) {
        while let Some(_) = self.user_data.dequeue() {}
        while let Some(_) = self.fn_data.dequeue() {}
    }
}

pub struct FnDataHandle<UD: 'static, FD: 'static, const N: usize> {
    producer: &'static MpMcQueue<FD, N>,
    consumer: &'static MpMcQueue<UD, N>,
}

impl<UD: 'static, FD: 'static, const N: usize> FnDataHandle<UD, FD, N> {
    pub fn new(producer: &'static MpMcQueue<FD, N>, consumer: &'static MpMcQueue<UD, N>) -> Self {
        Self { producer, consumer }
    }

    pub fn push(&self, data: FD) {
        self.producer.enqueue(data);
    }

    pub fn recv(&self) -> Option<UD> {
        self.consumer.dequeue()
    }
}

pub struct UserDataHandle<UD: 'static, FD: 'static, const N: usize> {
    producer: &'static MpMcQueue<UD, N>,
    consumer: &'static MpMcQueue<FD, N>,
}

impl<UD: 'static, FD: 'static, const N: usize> UserDataHandle<UD, FD, N> {
    pub fn new(producer: &'static MpMcQueue<UD, N>, consumer: &'static MpMcQueue<FD, N>) -> Self {
        Self { producer, consumer }
    }

    pub fn push(&self, data: UD) {
        self.producer.enqueue(data);
    }

    pub fn recv(&self) -> Option<FD> {
        self.consumer.dequeue()
    }
}
