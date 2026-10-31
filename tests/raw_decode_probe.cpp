// Standalone LibRaw capability probe. Never linked into Pivra.
#include <libraw/libraw.h>
#include <cstdint>
#include <filesystem>
#include <iostream>
#include <memory>

int main(int argc, char **argv) {
    if (argc != 2) {
        std::cerr << "Usage: raw_decode_probe <raw-file>\n";
        return 2;
    }
    namespace fs = std::filesystem;
    std::error_code ec;
    const fs::path path(argv[1]);
    const auto size = fs::file_size(path, ec);
    if (ec || !fs::is_regular_file(path) || size > 64ULL * 1024 * 1024) {
        std::cerr << "FAIL: missing, non-regular or larger than 64 MiB\n";
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
    raw.dcraw_clear_mem(img);
    if (!valid) {
        std::cerr << "FAIL invalid processed bitmap\n";
        return 1;
    }
    std::cout << "PASS LibRaw RAW decode. NOT a sandbox or Pivra integration test.\n";
    return 0;
}
