/* Execute the patched GTK CICP math in a real OpenGL context.
 * Readback is confined to assertions; no playback/frame conversion is provided.
 */
#define _POSIX_C_SOURCE 200809L
#include <epoxy/egl.h>
#include <epoxy/gl.h>
#include <glib.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <time.h>
#include "gdkcolordefs.h"
#include "gpu_layout.h"

/* EGL_EXT_device_drm_render_node, for older libepoxy development headers. */
#ifndef EGL_DRM_RENDER_NODE_FILE_EXT
#define EGL_DRM_RENDER_NODE_FILE_EXT 0x3377
#endif
#define N 264
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"FAIL %s:%d: %s\n",__FILE__,__LINE__,#x); exit(1); } } while (0)
static unsigned cases;
static double worst;
static GLuint input, target, framebuffer;
static uint64_t mono(void) { struct timespec t; CHECK(clock_gettime(CLOCK_MONOTONIC,&t)==0); return (uint64_t)t.tv_sec*1000000000+t.tv_nsec; }
static long rss(void) { struct rusage r; CHECK(getrusage(RUSAGE_SELF,&r)==0); return r.ru_maxrss; }
static char *read_source(const char *name) { char *s=NULL; GError *e=NULL; CHECK(g_file_get_contents(name,&s,NULL,&e)); return s; }
static GLuint shader(GLenum kind,const char *text) {
  GLuint s=glCreateShader(kind); GLint ok; glShaderSource(s,1,&text,NULL); glCompileShader(s); glGetShaderiv(s,GL_COMPILE_STATUS,&ok);
  if(!ok) { char log[8192]; glGetShaderInfoLog(s,sizeof log,NULL,log); fprintf(stderr,"%s\n",log); } CHECK(ok); return s;
}
static GLuint program(const char *vertex, const char *fragment) {
  GLuint v=shader(GL_VERTEX_SHADER,vertex),f=shader(GL_FRAGMENT_SHADER,fragment),p=glCreateProgram(); GLint ok;
  glAttachShader(p,v);glAttachShader(p,f);glLinkProgram(p);glGetProgramiv(p,GL_LINK_STATUS,&ok);
  if(!ok) { char log[8192]; glGetProgramInfoLog(p,sizeof log,NULL,log);fprintf(stderr,"%s\n",log); } CHECK(ok);
  glDeleteShader(v);glDeleteShader(f);return p;
}
static void ui(GLuint p,const char *name,unsigned value) { glUniform1ui(glGetUniformLocation(p,name),value); }
static void controls(GLuint p,unsigned prim,unsigned tf,unsigned matrix,unsigned range,unsigned requested,int reverse,int premul) {
  glUseProgram(p); ui(p,"u_primaries",prim);ui(p,"u_transfer",tf);ui(p,"u_matrix",matrix);ui(p,"u_range",range);ui(p,"u_requested",requested);
  glUniform1i(glGetUniformLocation(p,"u_reverse"),reverse);glUniform1i(glGetUniformLocation(p,"u_premul"),premul);glUniform1i(glGetUniformLocation(p,"u_tex"),0);
}
static void targets(int w,int h,int floating) {
  glBindTexture(GL_TEXTURE_2D,input);glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_MIN_FILTER,GL_NEAREST);glTexParameteri(GL_TEXTURE_2D,GL_TEXTURE_MAG_FILTER,GL_NEAREST);
  glBindTexture(GL_TEXTURE_2D,target);glTexImage2D(GL_TEXTURE_2D,0,floating?GL_RGBA32F:GL_RGBA16F,w,h,0,GL_RGBA,GL_FLOAT,NULL);
  glBindFramebuffer(GL_FRAMEBUFFER,framebuffer);glFramebufferTexture2D(GL_FRAMEBUFFER,GL_COLOR_ATTACHMENT0,GL_TEXTURE_2D,target,0);
  CHECK(glCheckFramebufferStatus(GL_FRAMEBUFFER)==GL_FRAMEBUFFER_COMPLETE);glViewport(0,0,w,h);glBindTexture(GL_TEXTURE_2D,input);
}
static void vectors(float data[N][4],unsigned matrix,unsigned range,int reverse,int premul) {
  static const float colors[8][3]={{1,0,0},{0,1,0},{0,0,1},{1,1,0},{0,1,1},{1,0,1},{0.01f,0.1f,0.7f},{0,0,0}};
  for(int i=0;i<N;i++) {
    float rgb[3];for(int c=0;c<3;c++) rgb[c]=i<256?i/255.f:colors[i-256][c];
    data[i][3]=premul?0.375f:1.f;
    if(premul && i==N-1) data[i][3]=0.f;
    if(matrix==1 && !reverse) {
      for(int c=0;c<3;c++) data[i][c]=rgb_to_bt709[3*c]*rgb[0]+rgb_to_bt709[3*c+1]*rgb[1]+rgb_to_bt709[3*c+2]*rgb[2]+(c==1?0.f:0.5f);
      if(!range) { data[i][0]=data[i][0]*224.f/255.f+16.f/255.f;data[i][1]=data[i][1]*219.f/255.f+16.f/255.f;data[i][2]=data[i][2]*224.f/255.f+16.f/255.f; }
    } else memcpy(data[i],rgb,sizeof rgb);
    if(premul && !reverse) for(int c=0;c<3;c++) data[i][c]*=data[i][3];
  }
}
static void draw(GLuint p,const float data[N][4],float out[N][4]) {
  glUseProgram(p);glBindTexture(GL_TEXTURE_2D,input);glTexImage2D(GL_TEXTURE_2D,0,GL_RGBA32F,N,1,0,GL_RGBA,GL_FLOAT,data);
  glDrawArrays(GL_TRIANGLES,0,3);glReadPixels(0,0,N,1,GL_RGBA,GL_FLOAT,out);CHECK(glGetError()==GL_NO_ERROR);
}
static void cpu_video(const float in[4],float out[4],unsigned matrix,unsigned range,int reverse,int premul) {
  memcpy(out,in,4*sizeof(float));
  if(reverse) {
    srgb_eotf(out);bt1886_oetf(out);
    if(matrix==1) {
      float rgb[3];memcpy(rgb,out,sizeof rgb);
      for(int c=0;c<3;c++) out[c]=rgb_to_bt709[3*c]*rgb[0]+rgb_to_bt709[3*c+1]*rgb[1]+rgb_to_bt709[3*c+2]*rgb[2]+(c==1?0.f:0.5f);
    }
    for(int c=0;c<3;c++) out[c]=CLAMP(out[c],0.f,1.f);
    if(!range) { out[0]=out[0]*224.f/255.f+16.f/255.f;out[1]=out[1]*219.f/255.f+16.f/255.f;out[2]=out[2]*224.f/255.f+16.f/255.f; }
    if(premul) for(int c=0;c<3;c++)out[c]*=out[3];
  } else {
    if(premul && out[3]>0) for(int c=0;c<3;c++)out[c]/=out[3];
    if(!range) { out[0]=CLAMP((out[0]-16.f/255.f)*255.f/224.f,0.f,1.f);out[1]=CLAMP((out[1]-16.f/255.f)*255.f/219.f,0.f,1.f);out[2]=CLAMP((out[2]-16.f/255.f)*255.f/224.f,0.f,1.f); }
    if(matrix==1) {
      float yuv[3]={out[0]-0.5f,out[1],out[2]-0.5f};
      for(int c=0;c<3;c++)out[c]=bt709_to_rgb[3*c]*yuv[0]+bt709_to_rgb[3*c+1]*yuv[1]+bt709_to_rgb[3*c+2]*yuv[2];
    }
    float encoded[3];memcpy(encoded,out,sizeof encoded);
    bt1886_eotf(out);
    for(int c=0;c<3;c++) CHECK(fabs(out[c]-pow(fmax(encoded[c],0.0),2.4))<0.000002);
    srgb_oetf(out);
  }
}
static void video_cases(GLuint patched,GLuint original) {
  float data[N][4],out[N][4],ref[4],old[N][4];targets(N,1,1);
  for(unsigned matrix=0;matrix<=1;matrix++) for(unsigned range=0;range<=1;range++) {
    if(matrix==0 && range==0) continue;
    for(int reverse=0;reverse<=1;reverse++)for(int premul=0;premul<=(matrix==0?1:0);premul++) {
      vectors(data,matrix,range,reverse,premul);controls(patched,1,1,matrix,range,1,reverse,premul);draw(patched,data,out);
      for(int i=0;i<N;i++) {
        cpu_video(data[i],ref,matrix,range,reverse,premul);
        for(int c=0;c<4;c++) {
          double error=fabs(out[i][c]-ref[c]);if(error>worst)worst=error;
          if(!isfinite(out[i][c]) || error>0.00003) fprintf(stderr,"video m=%u r=%u reverse=%d premul=%d pixel=%d c=%d gpu=%g cpu=%g error=%g\n",matrix,range,reverse,premul,i,c,out[i][c],ref[c],error);
          CHECK(isfinite(out[i][c]) && error<=0.00003);
        }
      }
      if(matrix==0 && !reverse && !premul) {
        controls(original,1,1,matrix,range,0,0,0);draw(original,data,old);
        CHECK(old[128][0]-out[128][0]>0.05f); // Demonstrate active EOTF change.
      }
      cases++;
    }
  }
}
static void unchanged_cases(GLuint patched,GLuint original) {
  const unsigned transfers[]={1,2,4,5,6,8,13,14,15,16,18};const unsigned primaries[]={1,9,12};
  float data[N][4],a[N][4],b[N][4];targets(N,1,1);vectors(data,0,1,0,0);
  for(unsigned prim=0;prim<G_N_ELEMENTS(primaries);prim++)for(unsigned t=0;t<G_N_ELEMENTS(transfers);t++) {
    unsigned cp=primaries[prim],tf=transfers[t];
    for(unsigned requested=0;requested<=1;requested++) {
      if(requested && cp==1 && tf==1)continue;
      controls(original,cp,tf,0,1,0,0,0);draw(original,data,a);
      controls(patched,cp,tf,0,1,requested,0,0);draw(patched,data,b);
      for(int i=0;i<N;i++)for(int c=0;c<4;c++) CHECK(isfinite(b[i][c]) && fabs(a[i][c]-b[i][c])<=0.00003);
      cases++;
    }
  }
}
static void bench(GLuint p,unsigned video,uint64_t *gpu,uint64_t *cpu) {
  const int w=1920,h=1080,frames=8;size_t bytes=(size_t)w*h*4;unsigned char *data=g_malloc(bytes);
  for(size_t i=0;i<bytes;i+=4) {data[i]=data[i+1]=data[i+2]=128;data[i+3]=255;}
  targets(w,h,0);glTexImage2D(GL_TEXTURE_2D,0,GL_RGBA8,w,h,0,GL_RGBA,GL_UNSIGNED_BYTE,data);g_free(data);
  controls(p,1,1,0,1,video,0,0);glDrawArrays(GL_TRIANGLES,0,3);glFinish(); // Warmup outside measurement.
  GLuint q;glGenQueries(1,&q);uint64_t begin=mono();glBeginQuery(GL_TIME_ELAPSED,q);
  for(int f=0;f<frames;f++)glDrawArrays(GL_TRIANGLES,0,3);
  glEndQuery(GL_TIME_ELAPSED);glFinish();*cpu=mono()-begin;
  GLuint64 elapsed;glGetQueryObjectui64v(q,GL_QUERY_RESULT,&elapsed);*gpu=elapsed;glDeleteQueries(1,&q);
  CHECK(glGetError()==GL_NO_ERROR);
}
static void json_string(const char *s) { putchar('"');for(;*s;s++){if(*s=='"'||*s=='\\')putchar('\\');if((unsigned char)*s>=32)putchar(*s);}putchar('"'); }
int main(int argc,char **argv) {
  if(argc!=5) {fprintf(stderr,"Usage: gpu_probe original.vert original.frag patched.vert patched.frag\n");return 2;}
  EGLDisplay display=eglGetDisplay(EGL_DEFAULT_DISPLAY);EGLint major,minor,n;
  if(display==EGL_NO_DISPLAY || !eglInitialize(display,&major,&minor)) {fprintf(stderr,"NOT RUN: EGL initialization\n");return 77;}
  const EGLint attrs[]={EGL_SURFACE_TYPE,EGL_PBUFFER_BIT,EGL_RENDERABLE_TYPE,EGL_OPENGL_BIT,EGL_RED_SIZE,8,EGL_GREEN_SIZE,8,EGL_BLUE_SIZE,8,EGL_NONE};EGLConfig config;
  if(!eglBindAPI(EGL_OPENGL_API)||!eglChooseConfig(display,attrs,&config,1,&n)||n!=1){fprintf(stderr,"NOT RUN: EGL/OpenGL config\n");return 77;}
  const EGLint ca[]={EGL_CONTEXT_MAJOR_VERSION,3,EGL_CONTEXT_MINOR_VERSION,3,EGL_CONTEXT_OPENGL_PROFILE_MASK,EGL_CONTEXT_OPENGL_CORE_PROFILE_BIT,EGL_NONE};
  const EGLint pa[]={EGL_WIDTH,1,EGL_HEIGHT,1,EGL_NONE};EGLSurface surface=eglCreatePbufferSurface(display,config,pa);EGLContext context=eglCreateContext(display,config,EGL_NO_CONTEXT,ca);
  if(context==EGL_NO_CONTEXT||surface==EGL_NO_SURFACE||!eglMakeCurrent(display,surface,surface,context)){fprintf(stderr,"NOT RUN: GL 3.3 context\n");return 77;}
  const char *vendor=(const char*)glGetString(GL_VENDOR),*renderer=(const char*)glGetString(GL_RENDERER),*version=(const char*)glGetString(GL_VERSION);
  CHECK(vendor&&renderer&&version&&epoxy_gl_version()>=33);
  const char *device="";EGLAttrib dev;
  PFNEGLQUERYDISPLAYATTRIBEXTPROC qd=(PFNEGLQUERYDISPLAYATTRIBEXTPROC)eglGetProcAddress("eglQueryDisplayAttribEXT");
  PFNEGLQUERYDEVICESTRINGEXTPROC qs=(PFNEGLQUERYDEVICESTRINGEXTPROC)eglGetProcAddress("eglQueryDeviceStringEXT");
  if(qd&&qs&&qd(display,EGL_DEVICE_EXT,&dev)){const char *d=qs((EGLDeviceEXT)dev,EGL_DRM_RENDER_NODE_FILE_EXT);if(d)device=d;}
  char *ov=read_source(argv[1]),*of=read_source(argv[2]),*nv=read_source(argv[3]),*nf=read_source(argv[4]);
  GLuint original=program(ov,of),patched=program(nv,nf);g_free(ov);g_free(of);g_free(nv);g_free(nf);
  GLuint vao;glGenVertexArrays(1,&vao);glBindVertexArray(vao);glGenTextures(1,&input);glGenTextures(1,&target);glGenFramebuffers(1,&framebuffer);
  long before=rss();video_cases(patched,original);unchanged_cases(patched,original);
  uint64_t a,b,c,d;bench(original,0,&a,&b);bench(patched,1,&c,&d);long peak=rss();
  printf("{\"vendor\":");json_string(vendor);printf(",\"renderer\":");json_string(renderer);printf(",\"gl_version\":");json_string(version);printf(",\"drm_render_node\":");json_string(device);
  printf(",\"instance_bytes_original\":%zu,\"instance_bytes_candidate\":%zu", sizeof(struct PivraOriginalInstance), sizeof(struct PivraCandidateInstance));
  printf(",\"shader_batches\":%u,\"vectors_per_batch\":%d,\"max_video_float_error\":%.9g,\"tolerance\":0.00003,\"bench_width\":1920,\"bench_height\":1080,\"bench_frames\":8,\"baseline_gpu_ns\":%"G_GUINT64_FORMAT",\"candidate_gpu_ns\":%"G_GUINT64_FORMAT",\"baseline_wall_ns\":%"G_GUINT64_FORMAT",\"candidate_wall_ns\":%"G_GUINT64_FORMAT",\"rss_before_kib\":%ld,\"peak_rss_kib\":%ld,\"bench_input_bytes\":8294400,\"bench_target_bytes\":16588800,\"readbacks_in_bench_loop\":0}\n",cases,N,worst,a,c,b,d,before,peak);
  glDeleteTextures(1,&input);glDeleteTextures(1,&target);glDeleteFramebuffers(1,&framebuffer);glDeleteVertexArrays(1,&vao);glDeleteProgram(original);glDeleteProgram(patched);
  eglMakeCurrent(display,EGL_NO_SURFACE,EGL_NO_SURFACE,EGL_NO_CONTEXT);eglDestroyContext(display,context);eglDestroySurface(display,surface);eglTerminate(display);return 0;
}
