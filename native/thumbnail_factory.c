/* Native GNOME factory adapter, not a new decoder. Source pixels are delegated
 * to GNOME's system thumbnail infrastructure; cache display still uses Glycin. */
#define _GNU_SOURCE
#define GNOME_DESKTOP_USE_UNSTABLE_API
#include <libgnome-desktop/gnome-desktop-thumbnail.h>
#include <gio/gio.h>
#include <sys/stat.h>
#include <sys/resource.h>
#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <time.h>
static int same(const struct stat *a, const struct stat *b) {
    return a->st_dev==b->st_dev && a->st_ino==b->st_ino && a->st_size==b->st_size &&
        a->st_mtim.tv_sec==b->st_mtim.tv_sec && a->st_mtim.tv_nsec==b->st_mtim.tv_nsec &&
        a->st_ctim.tv_sec==b->st_ctim.tv_sec && a->st_ctim.tv_nsec==b->st_ctim.tv_nsec;
}
int main(int argc, char **argv) {
    if (argc!=5 || strlen(argv[1])>16384 || strlen(argv[2])>256) return 2;
    if (strcmp(argv[4],"missing") && strcmp(argv[4],"stale")) return 2;
    struct rlimit memory={1024UL*1024*1024,1024UL*1024*1024};
    struct rlimit cpu={15,15}, output={4UL*1024*1024,4UL*1024*1024};
    if (setrlimit(RLIMIT_AS,&memory) || setrlimit(RLIMIT_CPU,&cpu) || setrlimit(RLIMIT_FSIZE,&output)) return 2;
    (void)nice(10);
    errno=0; char *end=NULL;
    gint64 mtime=g_ascii_strtoll(argv[3],&end,10);
    if (errno || !end || *end || mtime<0 || (gint64)(time_t)mtime!=mtime) return 2;
    struct timespec started;
    if (clock_gettime(CLOCK_REALTIME,&started)) return 2;
    GFile *file=g_file_new_for_uri(argv[1]);
    char *path=g_file_get_path(file);
    struct stat before,after;
    if (!path || stat(path,&before) || !S_ISREG(before.st_mode) || before.st_size>64*1024*1024 || before.st_mtime!=(time_t)mtime) return 2;
    GnomeDesktopThumbnailFactory *factory=gnome_desktop_thumbnail_factory_new(GNOME_DESKTOP_THUMBNAIL_SIZE_NORMAL);
    if (!factory) return 2;
    char *cached=gnome_desktop_thumbnail_factory_lookup(factory,argv[1],(time_t)mtime);
    if (cached && !strcmp(argv[4],"missing")) { g_free(cached); puts("cached"); return 0; }
    g_free(cached);
    if (!gnome_desktop_thumbnail_factory_can_thumbnail(factory,argv[1],argv[2],(time_t)mtime)) {
        puts("ineligible"); return 3;
    }
    GError *error=NULL;
    GdkPixbuf *thumbnail=gnome_desktop_thumbnail_factory_generate_thumbnail(factory,argv[1],argv[2],NULL,&error);
    if (!thumbnail) {
        printf("failed:%.160s\n",error?error->message:"factory returned no thumbnail");
        return 4;
    }
    if (gdk_pixbuf_get_width(thumbnail)>1024 || gdk_pixbuf_get_height(thumbnail)>1024 ||
        stat(path,&after) || !same(&before,&after)) { puts("source changed or output bound"); return 5; }
    if (!gnome_desktop_thumbnail_factory_save_thumbnail(factory,thumbnail,argv[1],(time_t)mtime,NULL,&error)) {
        printf("failed:%.160s\n",error?error->message:"cache save failed"); return 6;
    }
    /* Seconds-only Thumb::MTime cannot detect a same-second source replacement.
       Stamp the cache with generation START, not save completion: the provider's
       nanosecond source-ctime check then rejects even a change during save. */
    char *cache=gnome_desktop_thumbnail_path_for_uri(argv[1],GNOME_DESKTOP_THUMBNAIL_SIZE_NORMAL);
    int fd=cache?open(cache,O_RDONLY|O_NOFOLLOW|O_CLOEXEC):-1;
    struct stat output_stat;
    struct timespec times[2]={{.tv_sec=0,.tv_nsec=UTIME_OMIT},started};
    if (fd<0 || fstat(fd,&output_stat) || !S_ISREG(output_stat.st_mode) || output_stat.st_size>2*1024*1024 || futimens(fd,times)) return 6;
    if (stat(path,&after) || !same(&before,&after)) {
        /* Discard only the artifact we just saved, not a replacement written
           concurrently by another cache producer. Source files are never removed. */
        struct stat current;
        if (!lstat(cache,&current) && S_ISREG(current.st_mode) && current.st_dev==output_stat.st_dev && current.st_ino==output_stat.st_ino) (void)unlink(cache);
        close(fd); g_free(cache);
        puts("source changed during cache save"); return 5;
    }
    close(fd); g_free(cache);
    g_object_unref(thumbnail); g_object_unref(factory); g_object_unref(file); g_free(path);
    puts("generated"); return 0;
}
