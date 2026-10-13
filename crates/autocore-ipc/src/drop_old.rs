use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Tek yuvalı (single-slot) atomik posta kutusu.
/// Yeni bir öğe geldiğinde önceki okunmamış öğenin üzerine yazar (Drop-Old).
/// Otonom sistemlerde daima en güncel kararın/durumun işlenmesini garanti eder.
pub struct DropOldMailbox<T> {
    slot: Mutex<Option<T>>,
    dropped_count: AtomicU64,
}

impl<T> Default for DropOldMailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DropOldMailbox<T> {
    pub fn new() -> Self {
        Self {
            slot: Mutex::new(None),
            dropped_count: AtomicU64::new(0),
        }
    }

    /// Yeni veriyi posta kutusuna yazar.
    /// Eğer önceki veri henüz okunmamışsa, eski veriyi düşürür ve düşürülen öğeyi döndürür.
    pub fn send(&self, item: T) -> Option<T> {
        let mut guard = self.slot.lock().unwrap();
        let prev = guard.take();
        if prev.is_some() {
            self.dropped_count.fetch_add(1, Ordering::Relaxed);
        }
        *guard = Some(item);
        prev
    }

    /// Posta kutusundaki en güncel veriyi alır.
    pub fn recv(&self) -> Option<T> {
        let mut guard = self.slot.lock().unwrap();
        guard.take()
    }

    /// Posta kutusunda okunmamış veri olup olmadığını kontrol eder.
    pub fn is_empty(&self) -> bool {
        let guard = self.slot.lock().unwrap();
        guard.is_none()
    }

    /// Düşürülen (atlanmış) toplam paket sayısını verir.
    pub fn dropped_count(&self) -> u64 {
        self.dropped_count.load(Ordering::Relaxed)
    }
}

/// Sabit kapasiteli Drop-Old halka kuyruğu.
/// Kapasite dolduğunda en eski öğe kuyruktan atılarak yenisine yer açılır.
/// Bu sayede yavaş tüketiciler boru hattını şişiremez ve gecikme (latency) birikmez.
pub struct DropOldLeaseQueue<T> {
    capacity: usize,
    queue: Mutex<VecDeque<T>>,
    dropped_count: AtomicU64,
}

impl<T> DropOldLeaseQueue<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than zero");
        Self {
            capacity,
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            dropped_count: AtomicU64::new(0),
        }
    }

    /// Kuyruğa yeni öğe ekler.
    /// Kapasite aşılırsa en eski öğe çıkarılır ve `Some(dropped)` olarak döndürülür.
    pub fn push(&self, item: T) -> Option<T> {
        let mut guard = self.queue.lock().unwrap();
        let dropped = if guard.len() >= self.capacity {
            self.dropped_count.fetch_add(1, Ordering::Relaxed);
            guard.pop_front()
        } else {
            None
        };
        guard.push_back(item);
        dropped
    }

    /// Kuyruktan sıradaki en eski öğeyi çeker.
    pub fn pop(&self) -> Option<T> {
        let mut guard = self.queue.lock().unwrap();
        guard.pop_front()
    }

    pub fn len(&self) -> usize {
        let guard = self.queue.lock().unwrap();
        guard.len()
    }

    pub fn is_empty(&self) -> bool {
        let guard = self.queue.lock().unwrap();
        guard.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped_count.load(Ordering::Relaxed)
    }

    pub fn clear(&self) {
        let mut guard = self.queue.lock().unwrap();
        guard.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mailbox_drop_old() {
        let mailbox = DropOldMailbox::new();
        assert!(mailbox.is_empty());

        let dropped1 = mailbox.send(10);
        assert_eq!(dropped1, None);
        assert!(!mailbox.is_empty());

        // Okunmadan üzerine yaz: 10 düşmeli
        let dropped2 = mailbox.send(20);
        assert_eq!(dropped2, Some(10));
        assert_eq!(mailbox.dropped_count(), 1);

        // En son veri 20 olmalı
        assert_eq!(mailbox.recv(), Some(20));
        assert!(mailbox.is_empty());
        assert_eq!(mailbox.recv(), None);
    }

    #[test]
    fn test_queue_drop_old_capacity() {
        let queue = DropOldLeaseQueue::new(2);
        assert_eq!(queue.push(1), None);
        assert_eq!(queue.push(2), None);
        assert_eq!(queue.len(), 2);

        // 3 eklendiğinde en eski olan 1 düşmeli
        let dropped = queue.push(3);
        assert_eq!(dropped, Some(1));
        assert_eq!(queue.dropped_count(), 1);
        assert_eq!(queue.len(), 2);

        // Sıradakiler 2 ve 3 olmalı
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
    }
}
