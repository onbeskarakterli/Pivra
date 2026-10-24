/* Experimental child only. Launched by Bwrap, never linked into Pivra. */
#include <stdio.h>
#include <stdlib.h>
#include <jpeglib.h>
#include <sys/resource.h>
#include <time.h>
static void warning(j_common_ptr c, int level) {
    if (level < 0) { c->err->output_message(c); exit(2); }
}
int main(int argc, char **argv) {
    if (argc != 2) return 2;
    int mode = atoi(argv[1]);
    if (mode != 1 && mode != 8 && mode != 0) return 2;
    struct rlimit mem = {512UL*1024*1024,512UL*1024*1024}, cpu = {10,10};
    if (setrlimit(RLIMIT_AS,&mem) || setrlimit(RLIMIT_CPU,&cpu)) return 2;
    struct jpeg_decompress_struct c;
    struct jpeg_error_mgr err;
    c.err = jpeg_std_error(&err);
    err.emit_message = warning;
    jpeg_create_decompress(&c);
    c.mem->max_memory_to_use = 16UL*1024*1024;
    jpeg_stdio_src(&c,stdin);
    if (jpeg_read_header(&c,TRUE) != JPEG_HEADER_OK) return 2;
    if (!c.image_width || !c.image_height || c.image_width > 32768 || c.image_height > 32768 ||
        (unsigned long long)c.image_width*c.image_height > 32000000 || c.progressive_mode || c.arith_code) {
        fprintf(stderr,"UNSUPPORTED: requires bounded baseline Huffman JPEG\n"); return 3;
    }
    if (mode == 0 && (unsigned long long)c.image_width*c.image_height > 1048576) return 3;
    fprintf(stderr,"source=%ux%u progressive=0 scale=1/%d\n",c.image_width,c.image_height,mode==8?8:1);
    c.scale_num=1; c.scale_denom=mode==8?8:1; c.out_color_space=JCS_RGB;
    jpeg_start_decompress(&c);
    size_t row=(size_t)c.output_width*3, total=row*c.output_height;
    if (!row || !c.output_height || total > 96000000 || c.output_components != 3) return 2;
    unsigned char *buffer=malloc(total);
    if (!buffer) return 2;
    while(c.output_scanline<c.output_height) {
        JSAMPROW rows[1]={buffer+(size_t)c.output_scanline*row};
        if (jpeg_read_scanlines(&c,rows,1)!=1) return 2;
    }
    jpeg_finish_decompress(&c);
    fprintf(stderr,"output=%ux%u rgb_bytes=%zu\n",c.output_width,c.output_height,total);
    struct rusage usage;
    if (!getrusage(RUSAGE_SELF,&usage)) fprintf(stderr,"measured_peak_rss_kib=%ld\n",usage.ru_maxrss);
    printf("P6\n%u %u\n255\n",c.output_width,c.output_height);
    if (fwrite(buffer,1,total,stdout)!=total) return 2;
    free(buffer); jpeg_destroy_decompress(&c); return 0;
}
