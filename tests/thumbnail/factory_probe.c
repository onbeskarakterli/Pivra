/* Metadata capability only; never call generate_thumbnail or launch a decoder. */
#define GNOME_DESKTOP_USE_UNSTABLE_API
#include <libgnome-desktop/gnome-desktop-thumbnail.h>
#include <errno.h>
#include <stdio.h>
#include <sys/resource.h>
int main(int argc, char **argv) {
    if (argc!=4) return 2;
    struct rlimit memory={256UL*1024*1024,256UL*1024*1024};
    struct rlimit cpu={1,1}, output={0,0};
    if (setrlimit(RLIMIT_AS,&memory) || setrlimit(RLIMIT_CPU,&cpu) ||
        setrlimit(RLIMIT_FSIZE,&output)) return 2;
    char *end=NULL; errno=0;
    gint64 parsed=g_ascii_strtoll(argv[3],&end,10);
    if (errno || !end || *end || parsed<0 || (gint64)(time_t)parsed!=parsed) return 2;
    GnomeDesktopThumbnailFactory *factory=gnome_desktop_thumbnail_factory_new(GNOME_DESKTOP_THUMBNAIL_SIZE_NORMAL);
    if (!factory) return 2;
    gboolean supported=gnome_desktop_thumbnail_factory_can_thumbnail(factory,argv[1],argv[2],(time_t)parsed);
    printf("%d\n",supported?1:0);
    g_object_unref(factory); return 0;
}
