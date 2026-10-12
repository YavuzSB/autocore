//! autocore-safety :: Hayalet Hareket Koruması ve Atomik Kuyruk Temizleme (Phantom Movement Fix)

use std::collections::VecDeque;
use std::sync::Mutex;
use tracing::warn;

/// E-Stop ve SafeLock anında atomik olarak tamamen temizlenebilen komut kuyruğu
#[derive(Debug)]
pub struct LockFreeFlushQueue<T> {
    queue: Mutex<VecDeque<T>>,
    capacity: usize,
}

impl<T> LockFreeFlushQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        }
    }

    /// Kuyruğa yeni eleman ekler (Kapasite dolarsa en eskiyi atar - Drop-Old)
    pub fn push(&self, item: T) {
        let mut q = self.queue.lock().unwrap();
        if q.len() >= self.capacity {
            q.pop_front(); // En eskiyi düşür
        }
        q.push_back(item);
    }

    /// Kuyruktan sıradaki elemanı çeker
    pub fn pop(&self) -> Option<T> {
        let mut q = self.queue.lock().unwrap();
        q.pop_front()
    }

    /// En son (en taze) elemanı çeker, öncekileri atar
    pub fn pop_latest(&self) -> Option<T> {
        let mut q = self.queue.lock().unwrap();
        let latest = q.pop_back();
        q.clear(); // Eski tüm bayat paketleri temizle
        latest
    }

    /// Kuyruktaki eleman sayısını döndürür
    pub fn len(&self) -> usize {
        let q = self.queue.lock().unwrap();
        q.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// HAYALET HAREKET KORUMASI (Phantom Movement Fix):
    /// E-Stop veya SafeLock tetiklendiğinde kuyruktaki tüm bayat paketleri
    /// tek bir atomik kilit döngüsünde tamamen çöpe atar (flush).
    pub fn flush_all(&self) -> usize {
        let mut q = self.queue.lock().unwrap();
        let count = q.len();
        q.clear();
        if count > 0 {
            warn!(
                "HAYALET HAREKET KORUMASI: Kuyruktaki {} bekleyen komut çöpe atıldı (Flushed)!",
                count
            );
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autocore_base::KinematicCommand;

    #[test]
    fn test_phantom_movement_flush() {
        let queue = LockFreeFlushQueue::<KinematicCommand>::new(10);

        // Kuyruğa peş peşe 5 adet komut birikmiş olsun
        for i in 1..=5 {
            queue.push(KinematicCommand::new([i as f32, 0.0, 0.0], [0.0; 3], i));
        }
        assert_eq!(queue.len(), 5);

        // Acil durdurma anında flush çağrılır
        let flushed = queue.flush_all();
        assert_eq!(flushed, 5);
        assert_eq!(queue.len(), 0);

        // E-Stop kalktığında geçmişe dair hiçbir komut işletilemez
        assert!(queue.pop().is_none());
    }

    #[test]
    fn test_drop_old_on_capacity() {
        let queue = LockFreeFlushQueue::<i32>::new(3);

        queue.push(1);
        queue.push(2);
        queue.push(3);
        queue.push(4); // 1 düşmeli, [2, 3, 4] kalmalı

        assert_eq!(queue.len(), 3);
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), Some(4));
    }
}
