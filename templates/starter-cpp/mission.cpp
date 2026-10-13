/**
 * Autocore C++20 Başlangıç Görev Şablonu (Starter C++ Mission Node)
 * Sancak Otonom Sistemler (HSS, İKA, İDA, İHA) C++20 Düğümü
 */

#include "autocore.hpp"
#include <iostream>
#include <vector>
#include <chrono>
#include <thread>
#include <cmath>
#include <fcntl.h>
#include <unistd.h>
#include <termios.h>

int main(int argc, char* argv[]) {
    std::string port_path = "/dev/pts/1";
    if (argc > 1) {
        port_path = argv[1];
    }

    std::cout << "============================================================" << std::endl;
    std::cout << "🚀 SANCAK AUTOCORE C++20 OTONOM GÖREV DÜĞÜMÜ" << std::endl;
    std::cout << "============================================================" << std::endl;
    std::cout << "[CPP-MISSION] Port açılıyor: " << port_path << std::endl;

    int fd = open(port_path.c_str(), O_RDWR | O_NOCTTY);
    if (fd < 0) {
        std::cerr << "[CPP-MISSION] Port açılamadı: " << port_path << " (Hata: " << strerror(errno) << ")" << std::endl;
        std::cout << "[CPP-MISSION] Bilgi: Önce 'cargo run --bin autocore-sitl' ile SITL başlatın." << std::endl;
        return 1;
    }

    std::cout << "[CPP-MISSION] Bağlantı başarılı. Otonom devriye döngüsü başlıyor..." << std::endl;

    // 50 Hz döngüde 100 adım komut gönder
    for (int step = 1; step <= 100; ++step) {
        float vx = 2.0f;
        float yaw_rate = 0.1f * std::sin(step * 0.1f);
        uint64_t ts = std::chrono::duration_cast<std::chrono::microseconds>(
            std::chrono::steady_clock::now().time_since_epoch()
        ).count();

        auto cmd = autocore::KinematicCommand::make(vx, 0.0f, 0.0f, 0.0f, 0.0f, yaw_rate, ts);
        std::span<const uint8_t> payload(reinterpret_cast<const uint8_t*>(&cmd), sizeof(cmd));
        auto packet = autocore::encode_packet(autocore::MSG_TYPE_CMD_VEL, payload);

        write(fd, packet.data(), packet.size());

        // Telemetri oku
        uint8_t read_buf[256];
        ssize_t n = read(fd, read_buf, sizeof(read_buf));
        if (n > 0) {
            size_t consumed = 0;
            auto decoded = autocore::decode_packet(std::span<const uint8_t>(read_buf, n), consumed);
            if (decoded.has_value() && decoded->first == autocore::MSG_TYPE_FEEDBACK) {
                const auto* fb = reinterpret_cast<const autocore::ActuatorFeedback*>(decoded->second.data());
                if (step % 20 == 0) {
                    std::cout << "  [CPP-TELEMETRİ] Adım: " << step 
                              << " | Motor Akım 0: " << fb->motor_currents[0] << " A"
                              << " | Sıcaklık: " << fb->temperatures[0] << " °C" << std::endl;
                }
            }
        }

        std::this_thread::sleep_for(std::chrono::milliseconds(20)); // 50 Hz
    }

    // Görev sonu fren ve E-Stop
    auto stop_packet = autocore::encode_packet(autocore::MSG_TYPE_ESTOP, std::span<const uint8_t>{});
    write(fd, stop_packet.data(), stop_packet.size());
    close(fd);

    std::cout << "[CPP-MISSION] ✅ 100 Adımlık görev tamamlandı, port kapatıldı." << std::endl;
    return 0;
}
