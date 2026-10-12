# 🌉 BRAIN-KOPRU — SecondBrain'den Ne Çekilir

> **Bu dosya `KURALLAR.md` okuma sırasının parçasıdır (§3, madde 4).** Vault'un tamamını yükleme —
> aşağıdaki tabloda işine uyan tetik varsa ilgili dosyayı diskten oku, işine yarayan kesiti
> `docs/reports/`'a kopyala, altına `Kaynak: /home/yavuz/SecondBrain/...` yaz. Vault'a yazma, vault'u değiştirme.

## Köprü Tablosu

| İş / Tetik | SecondBrain'de Nerede | Ne Yap |
|---|---|---|
| Master Mimari Şartnamesi | `docs/ARCHITECTURE-AUTOCORE.md` | Projenin 6 fazlı teknik sözleşmesini, C-ABI hizalamalarını ve kural setini kontrol et |
| Orijinal Mimari Taslak & PDF Metni | `inbox/cekirdek.mimari.md` | Gereksinim veya sistem profili (HSS, İKA, İDA, İHA) detayına bakılacaksa incele |
| Büyük Proje & Vibe Coding İlkeleri | `knowledge/vibe-coding-buyuk-proje-sistemi.md` | 800 satır sınırı, modüler crate izolasyonu, 10→8 dokümantasyon kuralını hatırla |
| Doğrulama Kontrol Listesi | `skills/dogrulama-kontrol-listesi/SKILL.md` | Her faz sonu test, linter, derleme ve CI doğrulama adımlarını çıkar |
| Proje Akışı & Hibrit Damıtma | `skills/proje-akisi-hibrit-damitma.md` | Faz tamamlandığında dersleri damıt, `KALDIGIMIZ_YER.md` ve `SISTEM_RAPORU` güncelle |
| Vault İçi Semantik / Vektör Arama | `skills/vector-arama.md` | `python scripts/vector-query.py "..."` (SecondBrain dizininde) |

## Çekme Kuralı (3 Adım)

1. Tetik eşleştiyse dosyayı mutlak yoldan oku: `/home/yavuz/SecondBrain/<yol>`.
2. Kesiti (5–10 satır, ham kopya değil) ilgili `docs/reports/*.md`'ye veya faz notuna al.
3. Altına `Kaynak: /home/yavuz/SecondBrain/<dosya>` yaz. Proje klasörü tek başına bağımsız ve anlamlı kalmalıdır.

## Girmez (Yasak)

- Proje sırrı, takımların donanım pinleri veya hassas telemetri SecondBrain'e **geçmez** — sadece genellenebilir mühendislik dersi ve mimari karar.
- Bütün dosya kopyalama yok — o an gereksinim duyulan mimari kesit.

---
*Kurulum: 2026-10-13 — Autocore SecondBrain Entegrasyon Köprüsü. Tabloya yeni satır eklenebilir, temel yapı değişmez.*
