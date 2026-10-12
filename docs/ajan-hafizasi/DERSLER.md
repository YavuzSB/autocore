---
title: DERSLER.md
repo: autocore
guncelleme: 2026-10-13
---

# DERSLER.md

- 2026-10-13 [Linux PTY & Binary Serial Framing]: `libc::openpty` çağrısı Linux'ta varsayılan olarak kanonik terminal modunu (`ICANON | ECHO | ISIG`) etkin kılar. İkili protokol paketlerindeki `0x03` (ETX), `0x0A` (LF) gibi baytlar terminal sürücüsü tarafından yutulur veya `SIGINT` sinyaline çevrilir. Hem master hem slave PTY dosya tanıtıcıları üzerinde derhal `libc::cfmakeraw` çalıştırılarak bayt şeffaflığı (`raw mode`) sağlanmalıdır.
- 2026-10-13 [Tokio File Async I/O ve Task Bölme]: `tokio::fs::File` işletim sisteminin epoll/kqueue mekanizmasıyla doğrudan asenkron çalışan bir soket değildir; arkasında thread havuzundan `spawn_blocking` çalıştırır. Tek bir PTY dosya tanıtıcısını `tokio::select!` bloğu içinde hem okuma hem yazma için paylaşmak deadlock veya gecikme yaratır. Okuma ve yazma döngüleri iki ayrı bağımsız Tokio görevine (`tokio::spawn`) bölünmelidir.
- 2026-10-13 [Rust f64 PartialEq vs Eq]: Kayan noktalı sayılar (`f64`) IEEE 754 standardındaki `NaN` (Not a Number) değeri nedeniyle `Eq` uygulamaz (`NaN != NaN`). Koordinat taşıyan enum'larda (Örn: `GeofenceBreach { x: f64, y: f64, z: f64 }`) `#[derive(PartialEq)]` türetilmeli, `Eq` türetilmemelidir.
- 2026-10-13 [Hayalet Hareket (Phantom Movement) Riski]: Donanımsal acil durdurma (E-Stop) butonuna basıldığında motorlar dursa bile işletim sistemi soketlerinde veya yazılımsal kuyruklarda birikmiş eski hareket emirleri kalabilir. E-Stop butonu serbest bırakıldığında bu birikmiş komutlar peş peşe işletilerek aracı ileri fırlatır. E-Stop anında ve SafeLock geçişinde `LockFreeFlushQueue::flush_all()` çağrılarak kuyruk atomik olarak boşaltılmalıdır.
- 2026-10-13 [C-ABI ve static_assertions]: Farklı diller (Rust, C++, Python) arasında `/dev/shm` veya soket üzerinden struct paylaşırken her iki tarafın bellek dizilimi (struct layout & padding) birebir eşleşmelidir. Rust tarafında `#[repr(C)]` kullanılmalı ve `static_assertions::assert_eq_size!` ile derleme anında struct boyutları kilitlenmelidir.
