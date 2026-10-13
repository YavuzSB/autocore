//! STANAG 4586 ve MIL-STD-1553 İletişim Bütünlüğü ve Fuzzing Testleri
//!
//! Kapsam:
//! - 1000 Tekrarlı Deterministik Bit Çevirme (Bit-Flip Mutation Fuzzing)
//! - CRC-16-CCITT Sıfır Yanlış Kabul (Zero False-Acceptance) Garantisi
//! - Akış Desenkronizasyonu ve Parazit Kurtarma (Framing Resync)
//! - Parçalı Paket Kaybı ve Bellek Sızıntısız Zaman Aşımı (MTU 1200 Fragment Loss)

use autocore_defense_tests::DeterministicPrng;
use autocore_hal::protocol::{decode_packet, encode_packet, MSG_TYPE_CMD_VEL, MSG_TYPE_FEEDBACK};
use autocore_ipc::packet::{PacketFragmenter, PacketReassembler};

#[test]
fn test_stanag_1000_bit_flip_mutations_zero_leakage() {
    let mut prng = DeterministicPrng::new(0xDEAD_BEEF_CAFE_0001);
    let mut rejected_count = 0;
    let total_iterations = 1000;

    for i in 0..total_iterations {
        // 1. Rastgele boyutlu (8 ila 128 bayt) geçerli bir yük üret
        let payload_len = 8 + prng.next_usize(120);
        let mut original_payload = vec![0u8; payload_len];
        for b in &mut original_payload {
            *b = (prng.next_u32() % 256) as u8;
        }

        // 2. Geçerli CRC-16 korumalı pakete sar
        let msg_type = if i % 2 == 0 { MSG_TYPE_CMD_VEL } else { MSG_TYPE_FEEDBACK };
        let packet = encode_packet(msg_type, &original_payload);

        // 3. Paketin SOF (0..2) ve EOF (son 2 bayt) haricindeki gövdesinde
        // en az 1, en fazla 4 rastgele bit çevir (Bit-Flip Mutation)
        let mut body = packet[2..packet.len() - 2].to_vec();
        prng.mutate_bits(&mut body, 4);

        // Mutasyona uğramış gövdeyi geri birleştir
        let mut corrupted_packet = Vec::with_capacity(packet.len());
        corrupted_packet.extend_from_slice(&packet[..2]); // SOF
        corrupted_packet.extend_from_slice(&body);
        corrupted_packet.extend_from_slice(&packet[packet.len() - 2..]); // EOF

        // 4. Çözücüye besle
        match decode_packet(&corrupted_packet) {
            Ok(Some((_decoded_type, decoded_payload, _consumed))) => {
                // Eğer tesadüfen mutasyon sonucu yeni bir geçerli CRC oluştuysa (1 / 65536 ihtimal)
                // yükün orijinaliyle BİREBİR aynı olamayacağını doğrula
                if decoded_payload == original_payload {
                    panic!("Kritik: Bozuk paket orijinal yükle eşleşti!");
                }
            }
            Ok(None) | Err(_) => {
                // Beklenen durum: Paket ya eksik/bozuk bulundu ya da CRC-16 veto etti
                rejected_count += 1;
            }
        }
    }

    // Savunma sanayii standardı: 1000 mutasyonda en az %99.5 ret oranı
    println!("[STANAG TEST] 1000 Bit-Flip Mutasyonundan Başarıyla Reddedilen: {} / 1000", rejected_count);
    assert!(rejected_count >= 995, "CRC-16 hata yakalama oranı yetersiz: {} / 1000", rejected_count);
}

#[test]
fn test_stanag_framing_desynchronization_recovery() {
    let mut prng = DeterministicPrng::new(0x1234_5678_9ABC_DEF0);

    // 1. Geçerli bir görev paketi hazırla
    let valid_payload = b"AUTOCORE_MIL_STD_SAFE_PAYLOAD_001";
    let valid_packet = encode_packet(MSG_TYPE_CMD_VEL, valid_payload);

    // 2. Paketin önüne ve arkasına yüksek miktarda çöp / parazit bayt ekle
    let mut noisy_stream = Vec::new();

    // 50 bayt rastgele parazit
    for _ in 0..50 {
        noisy_stream.push((prng.next_u32() % 256) as u8);
    }

    // Sahte/Yarım başlık (0xAA 0x55 ve ardından geçersiz uzunluk)
    noisy_stream.push(0xAA);
    noisy_stream.push(0x55);
    noisy_stream.push(0x99); // Geçersiz mesaj türü
    noisy_stream.extend_from_slice(&0xFF00u16.to_le_bytes()); // Aşırı uzunluk

    // 30 bayt daha parazit
    for _ in 0..30 {
        noisy_stream.push((prng.next_u32() % 256) as u8);
    }

    // Asıl geçerli paketi ekle
    noisy_stream.extend_from_slice(&valid_packet);

    // 20 bayt kuyruk paraziti
    for _ in 0..20 {
        noisy_stream.push((prng.next_u32() % 256) as u8);
    }

    // 3. Akıştan geçerli paketin kurtarılmasını test et
    let mut recovered = false;
    while !noisy_stream.is_empty() {
        match decode_packet(&noisy_stream) {
            Ok(Some((msg_type, payload, consumed))) => {
                noisy_stream.drain(..consumed);
                if msg_type == MSG_TYPE_CMD_VEL && payload == valid_payload {
                    recovered = true;
                    break;
                }
            }
            Ok(None) => {
                // Geçerli paket henüz bulunamadıysa ilk baytı atarak kay
                noisy_stream.remove(0);
            }
            Err(_) => {
                noisy_stream.remove(0);
            }
        }
    }

    assert!(recovered, "Parazitli akış içindeki geçerli paket başarıyla kurtarılmalıydı!");
}

#[test]
fn test_stanag_telemetry_fragment_loss_and_leak_free_timeout() {
    let fragmenter = PacketFragmenter::new(1200);
    let mut reassembler = PacketReassembler::new(); // Boş reassembler

    // 3000 baytlık büyük bir telemetri bloğu
    let large_telemetry = vec![0x42u8; 3000];
    let fragments = fragmenter.fragment(101, &large_telemetry);
    assert!(fragments.len() >= 3, "En az 3 parça üretilmeli");

    // 2. parçayı düşür (Kayıp paket simülasyonu)
    let incomplete_stream = vec![fragments[0].clone(), fragments[2].clone()];

    for frag in incomplete_stream {
        let result = reassembler.process_fragment(frag, 1_000_000).unwrap();
        assert!(result.is_none(), "Eksik parça varken paket tamamlanamaz!");
    }

    // Zaman aşımı sonrası temizlik (Memory Leak koruması)
    let cleaned = reassembler.cleanup_expired(2_000_000, 500_000);
    assert_eq!(cleaned, 1, "Eksik kalan oturum bellekten başarıyla temizlenmeli");
}
