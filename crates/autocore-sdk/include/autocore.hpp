#pragma once

/**
 * Autocore C++20 Header-Only C-ABI Wrapper
 * 
 * AutoCore Autonomous Robotics Runtime & Safety Kernel
 * Zero-Copy binary layout and C-ABI compatibility layer for C++20 nodes.
 */

#include <cstdint>
#include <cstring>
#include <array>
#include <span>
#include <vector>
#include <optional>
#include <cassert>
#include <string>

namespace autocore {

// ============================================================================
// 1. Sabitler ve Protokol Tanımları
// ============================================================================

constexpr uint8_t SOF_BYTE_1 = 0xAA;
constexpr uint8_t SOF_BYTE_2 = 0x55;
constexpr uint8_t EOF_BYTE_1 = 0x0D;
constexpr uint8_t EOF_BYTE_2 = 0x0A;

constexpr uint8_t MSG_TYPE_CMD_VEL = 0x01;
constexpr uint8_t MSG_TYPE_FEEDBACK = 0x02;
constexpr uint8_t MSG_TYPE_ESTOP = 0x03;
constexpr uint8_t MSG_TYPE_HEARTBEAT = 0x04;

enum class SafetyMode : uint8_t {
    Booting = 0,
    Disarmed = 1,
    Armed = 2,
    Mission = 3,
    Failsafe = 4,
    SafeLock = 5,
    ManualOverride = 6
};

// ============================================================================
// 2. C-ABI Uyumlu Veri Yapıları (POD & Zero-Copy)
// ============================================================================

#pragma pack(push, 8)

/**
 * Standart Kinematik Hız Komutu (32 Bayt)
 * V = [Vx, Vy, Vz, RollRate, PitchRate, YawRate]^T
 */
struct KinematicCommand {
    float linear[3];       // [Vx, Vy, Vz] (m/s)
    float angular[3];      // [RollRate, PitchRate, YawRate] (rad/s)
    uint64_t timestamp_us; // Monotonic zaman damgası (mikrosaniye)

    static KinematicCommand make(float vx, float vy, float vz, float roll, float pitch, float yaw, uint64_t ts = 0) {
        KinematicCommand cmd{};
        cmd.linear[0] = vx; cmd.linear[1] = vy; cmd.linear[2] = vz;
        cmd.angular[0] = roll; cmd.angular[1] = pitch; cmd.angular[2] = yaw;
        cmd.timestamp_us = ts;
        return cmd;
    }
};
static_assert(sizeof(KinematicCommand) == 32, "KinematicCommand must be exactly 32 bytes");
static_assert(alignof(KinematicCommand) == 8, "KinematicCommand must be 8-byte aligned");

/**
 * 6 Serbestlik Dereceli Durum Vektörü (64 Bayt)
 */
struct StateVector6DoF {
    float position[3];      // [X, Y, Z] (m)
    float velocity[3];      // [Vx, Vy, Vz] (m/s)
    float quaternion[4];    // [W, X, Y, Z] (Hamilton norm = 1.0)
    float angular_rates[3]; // [P, Q, R] (rad/s)
    uint8_t _pad[4];        // 4 bayt hizalama dolgusu
    uint64_t timestamp_us;  // Mikrosaniye zaman damgası
};
static_assert(sizeof(StateVector6DoF) == 64, "StateVector6DoF must be exactly 64 bytes");
static_assert(alignof(StateVector6DoF) == 8, "StateVector6DoF must be 8-byte aligned");

/**
 * Aktüatör ve Donanım Telemetri Geri Beslemesi (72 Bayt)
 */
struct ActuatorFeedback {
    float actual_linear[3];      // Gerçekleşen doğrusal hızlar [Vx, Vy, Vz] (m/s)
    float actual_angular[3];     // Gerçekleşen açısal hızlar [Roll, Pitch, Yaw] (rad/s)
    float motor_currents[4];     // Motor akımları [M1, M2, M3, M4] (A)
    float temperatures[4];       // Sıcaklıklar [S1, S2, S3, S4] (°C)
    uint32_t status_flags;       // Durum bayrakları (bitmask)
    uint32_t _pad;               // 8-bayt hizalama dolgusu
    uint64_t timestamp_us;       // Zaman damgası
};
static_assert(sizeof(ActuatorFeedback) == 72, "ActuatorFeedback must be exactly 72 bytes");
static_assert(alignof(ActuatorFeedback) == 8, "ActuatorFeedback must be 8-byte aligned");

/**
 * Siyah Kutu Dosya Başlığı (40 Bayt)
 */
struct BlackboxHeader {
    uint8_t magic[4];       // "BBOX"
    uint32_t version;       // 1
    uint32_t capacity;      // Tampon kapasitesi
    uint32_t record_size;   // 184
    uint64_t total_records; // Toplam kaydedilen kare
    uint64_t start_time_us; // Başlangıç zamanı
    uint64_t end_time_us;   // Bitiş zamanı
};
static_assert(sizeof(BlackboxHeader) == 40, "BlackboxHeader must be exactly 40 bytes");
static_assert(alignof(BlackboxHeader) == 8, "BlackboxHeader must be 8-byte aligned");

/**
 * Siyah Kutu Telemetri Çerçevesi (184 Bayt)
 */
struct BlackboxFrame {
    uint64_t timestamp_us;
    StateVector6DoF state;
    KinematicCommand command;
    ActuatorFeedback feedback;
    uint16_t crc16;
    uint8_t _padding[6];
};
static_assert(sizeof(BlackboxFrame) == 184, "BlackboxFrame must be exactly 184 bytes");
static_assert(alignof(BlackboxFrame) == 8, "BlackboxFrame must be 8-byte aligned");

#pragma pack(pop)

// ============================================================================
// 3. CRC-16 ve Protokol Çerçeveleme Fonksiyonları
// ============================================================================

/**
 * Standart CRC-16-CCITT (Polynom: 0x1021, Başlangıç: 0xFFFF)
 */
inline uint16_t crc16_ccitt(std::span<const uint8_t> data) {
    uint16_t crc = 0xFFFF;
    for (uint8_t byte : data) {
        crc ^= static_cast<uint16_t>(byte) << 8;
        for (int i = 0; i < 8; ++i) {
            if ((crc & 0x8000) != 0) {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    return crc;
}

/**
 * Verilen mesaj tipi ve yükü CRC-16 korumalı pakete sarar
 * [0xAA 0x55][MSG_TYPE(1B)][LEN(2B LE)][PAYLOAD(NB)][CRC(2B LE)][0x0D 0x0A]
 */
inline std::vector<uint8_t> encode_packet(uint8_t msg_type, std::span<const uint8_t> payload) {
    const uint16_t len = static_cast<uint16_t>(payload.size());
    std::vector<uint8_t> packet;
    packet.reserve(2 + 1 + 2 + payload.size() + 2 + 2);

    packet.push_back(SOF_BYTE_1);
    packet.push_back(SOF_BYTE_2);
    packet.push_back(msg_type);
    packet.push_back(static_cast<uint8_t>(len & 0xFF));
    packet.push_back(static_cast<uint8_t>((len >> 8) & 0xFF));

    packet.insert(packet.end(), payload.begin(), payload.end());

    // CRC MSG_TYPE + LEN + PAYLOAD üzerinden hesaplanır
    const uint16_t crc = crc16_ccitt(std::span<const uint8_t>(packet.data() + 2, packet.size() - 2));
    packet.push_back(static_cast<uint8_t>(crc & 0xFF));
    packet.push_back(static_cast<uint8_t>((crc >> 8) & 0xFF));

    packet.push_back(EOF_BYTE_1);
    packet.push_back(EOF_BYTE_2);

    return packet;
}

/**
 * Gelen ikili bayt dizisinden geçerli paketi ayrıştırır
 */
inline std::optional<std::pair<uint8_t, std::vector<uint8_t>>> decode_packet(
    std::span<const uint8_t> buf, size_t& consumed
) {
    consumed = 0;
    if (buf.size() < 9) { // 2 SOF + 1 TYPE + 2 LEN + 2 CRC + 2 EOF
        return std::nullopt;
    }

    size_t sof_idx = 0;
    bool found_sof = false;
    for (size_t i = 0; i + 1 < buf.size(); ++i) {
        if (buf[i] == SOF_BYTE_1 && buf[i + 1] == SOF_BYTE_2) {
            sof_idx = i;
            found_sof = true;
            break;
        }
    }

    if (!found_sof) {
        consumed = buf.size();
        return std::nullopt;
    }

    const size_t remaining = buf.size() - sof_idx;
    if (remaining < 9) {
        consumed = sof_idx;
        return std::nullopt;
    }

    const uint8_t msg_type = buf[sof_idx + 2];
    const uint16_t payload_len = static_cast<uint16_t>(buf[sof_idx + 3]) |
                                (static_cast<uint16_t>(buf[sof_idx + 4]) << 8);

    const size_t total_packet_len = 2 + 1 + 2 + payload_len + 2 + 2;
    if (remaining < total_packet_len) {
        consumed = sof_idx;
        return std::nullopt;
    }

    const size_t eof_pos = sof_idx + total_packet_len - 2;
    if (buf[eof_pos] != EOF_BYTE_1 || buf[eof_pos + 1] != EOF_BYTE_2) {
        consumed = sof_idx + 2;
        return std::nullopt;
    }

    const size_t crc_pos = eof_pos - 2;
    const uint16_t expected_crc = static_cast<uint16_t>(buf[crc_pos]) |
                                 (static_cast<uint16_t>(buf[crc_pos + 1]) << 8);

    const uint16_t computed_crc = crc16_ccitt(
        std::span<const uint8_t>(buf.data() + sof_idx + 2, 1 + 2 + payload_len)
    );

    if (computed_crc != expected_crc) {
        consumed = sof_idx + 2;
        return std::nullopt;
    }

    std::vector<uint8_t> payload(
        buf.data() + sof_idx + 5,
        buf.data() + sof_idx + 5 + payload_len
    );

    consumed = sof_idx + total_packet_len;
    return std::make_pair(msg_type, std::move(payload));
}

} // namespace autocore
