// Isolated LibRaw decoder executable. Never linked into Pivra's GTK process.
#include <libraw/libraw.h>
#include <cstdint>
#include <filesystem>
#include <iostream>
#include <fstream>
#include <string>
#include <memory>

int main(int argc, char **argv) {
    if (argc != 2 && argc != 3) {
        std::cerr << "Usage: raw_decode_probe <raw-file> [output-ppm]\n";
        return 2;
    }
    namespace fs = std::filesystem;
    std::error_code ec;
    const fs::path path(argv[1]);
    const auto size = fs::file_size(path, ec);
    if (ec || !fs::is_regular_file(path) || size == 0 || size > 64ULL * 1024 * 1024) {
        std::cerr << "FAIL: empty, missing, non-regular or larger than 64 MiB\n";
        return 2;
    }
    LibRaw raw;
    auto check = [&](int code, const char *stage) {
        if (code != LIBRAW_SUCCESS) {
            std::cerr << "FAIL " << stage << ": " << libraw_strerror(code) << " (" << code << ")\n";
            return false;
        }
        std::cout << "PASS " << stage << "\n";
        return true;
    };
    if (!check(raw.open_file(argv[1]), "open_file")) return 1;
    if (!check(raw.unpack(), "unpack")) return 1;
    if (!check(raw.dcraw_process(), "dcraw_process")) return 1;
    int err = 0;
    auto *img = raw.dcraw_make_mem_image(&err);
    if (!img) {
        std::cerr << "FAIL make_mem_image: " << libraw_strerror(err) << "\n";
        return 1;
    }
    const bool valid = img->type == LIBRAW_IMAGE_BITMAP &&
        img->width > 0 && img->height > 0 && img->colors >= 3 &&
        (img->bits == 8 || img->bits == 16) && img->data_size > 0;
    std::cout << "BITMAP width=" << img->width << " height=" << img->height
              << " colors=" << img->colors << " bits=" << img->bits
              << " bytes=" << img->data_size << "\n";
    if (valid && argc == 3) {
        const uint64_t expected = uint64_t(img->width) * img->height * img->colors * (img->bits / 8);
        if (img->bits != 8 || img->colors != 3 || expected != img->data_size ||
            expected > 256ULL * 1024 * 1024 || img->width > 16384 || img->height > 16384) {
            std::cerr << "FAIL: unsafe PPM output geometry or size\n";
            raw.dcraw_clear_mem(img);
            return 1;
        }
        std::ofstream out(argv[2], std::ios::binary | std::ios::trunc);
        out << "P6\n" << img->width << " " << img->height << "\n255\n";
        out.write(reinterpret_cast<const char *>(img->data), img->data_size);
        out.flush();
        if (!out.good()) {
            std::cerr << "FAIL: PPM write error\n";
            raw.dcraw_clear_mem(img);
            return 1;
        }
        std::cout << "PASS bounded PPM transfer output\n";
    }
    raw.dcraw_clear_mem(img);
    if (!valid) {
        std::cerr << "FAIL invalid processed bitmap\n";
        return 1;
    }
    std::cout << "PASS LibRaw RAW decode. NOT a sandbox or Pivra integration test.\n";
    return 0;
}
