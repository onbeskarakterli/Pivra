/*
 * Copyright © 2018 Benjamin Otte
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library. If not, see <http://www.gnu.org/licenses/>.
 *
 * Authors: Benjamin Otte <otte@gnome.org>
 */

/* Application-owned playbin controller. No GstPlayer/GstPlay helper lifetime. */
#include "config.h"
#include "gtkgstmediafileprivate.h"
#include "gtkgstpaintableprivate.h"
#include <gst/gst.h>
#include "lifecycle.h"
#include "playback_rate.h"
struct _PivraGpuMediaFile {
  GtkMediaFile parent_instance;
  GdkPaintable *paintable;
  GstElement *pipeline;
  guint bus_watch, tick;
  gboolean audio, video, seeking;
  gint64 seek_target, active_target;
  GstPad *video_pad;
  guint rate, pending_rate;
  gboolean pitch_filter;
};
enum { PROP_0, PROP_RATE, PROP_RATE_PENDING, N_PROPERTIES };
static GParamSpec *properties[N_PROPERTIES];
static void close_file(GtkMediaFile *file);
static void pipeline_retired(gpointer unused, GObject *object) { (void)unused;pivra_video_lifecycle("pipeline-retired",object); }
static void seek(GtkMediaStream *stream,gint64 timestamp);
static void snapshot(GdkPaintable *p,GdkSnapshot *s,double w,double h) { gdk_paintable_snapshot(PIVRA_GPU_MEDIA_FILE(p)->paintable,s,w,h); }
static GdkPaintable *current_image(GdkPaintable *p) { return gdk_paintable_get_current_image(PIVRA_GPU_MEDIA_FILE(p)->paintable); }
static int width(GdkPaintable *p) { return gdk_paintable_get_intrinsic_width(PIVRA_GPU_MEDIA_FILE(p)->paintable); }
static int height(GdkPaintable *p) { return gdk_paintable_get_intrinsic_height(PIVRA_GPU_MEDIA_FILE(p)->paintable); }
static double aspect(GdkPaintable *p) { return gdk_paintable_get_intrinsic_aspect_ratio(PIVRA_GPU_MEDIA_FILE(p)->paintable); }
static void paintable_init(GdkPaintableInterface *iface) {
  iface->snapshot=snapshot;iface->get_current_image=current_image;
  iface->get_intrinsic_width=width;iface->get_intrinsic_height=height;iface->get_intrinsic_aspect_ratio=aspect;
}
G_DEFINE_TYPE_WITH_CODE(PivraGpuMediaFile,pivra_gpu_media_file,GTK_TYPE_MEDIA_FILE,
  G_IMPLEMENT_INTERFACE(GDK_TYPE_PAINTABLE,paintable_init))
static GWeakRef *weak_new(gpointer object) { GWeakRef *w=g_new0(GWeakRef,1);g_weak_ref_init(w,object);return w; }
static void weak_free(gpointer data) { GWeakRef *w=data;g_weak_ref_clear(w);g_free(w); }
static void update(PivraGpuMediaFile *self) {
  gint64 position;
  if(self->pipeline && gtk_media_stream_is_prepared(GTK_MEDIA_STREAM(self)) &&
     gst_element_query_position(self->pipeline,GST_FORMAT_TIME,&position) && position>=0)
    gtk_media_stream_update(GTK_MEDIA_STREAM(self),position/GST_USECOND);
}
static gboolean tick(gpointer data) {
  PivraGpuMediaFile *self=g_weak_ref_get(data);
  if(!self) return G_SOURCE_REMOVE;
  update(self);g_object_unref(self);return G_SOURCE_CONTINUE;
}
static void prepared(PivraGpuMediaFile *self) {
  if(!self->pipeline || gtk_media_stream_is_prepared(GTK_MEDIA_STREAM(self))) return;
  gint64 duration=0;gboolean seekable=FALSE;
  GstQuery *query=gst_query_new_seeking(GST_FORMAT_TIME);
  if(gst_element_query(self->pipeline,query)) gst_query_parse_seeking(query,NULL,&seekable,NULL,NULL);
  gst_query_unref(query);
  if(!gst_element_query_duration(self->pipeline,GST_FORMAT_TIME,&duration) || duration<0) duration=0;
  gtk_media_stream_stream_prepared(GTK_MEDIA_STREAM(self),self->audio,self->video,seekable,duration/GST_USECOND);
}
static void failure(PivraGpuMediaFile *self,GstMessage *message) {
  GError *error=NULL;gchar *debug=NULL;
  gst_message_parse_error(message,&error,&debug);
  g_printerr("Pivra GPU pipeline: %s (%s)\n",error->message,debug ? debug : "");
  if(!gtk_media_stream_get_error(GTK_MEDIA_STREAM(self))) gtk_media_stream_gerror(GTK_MEDIA_STREAM(self),error);
  else g_error_free(error);
  g_free(debug);
  close_file(GTK_MEDIA_FILE(self));
}

gboolean pivra_gpu_has_pitch_filter(GtkMediaStream *stream)
{
  return PIVRA_IS_GPU_MEDIA_FILE(stream) && PIVRA_GPU_MEDIA_FILE(stream)->pitch_filter;
}

gboolean pivra_gpu_set_playback_rate(GtkMediaStream *stream, guint quarters)
{
  if (!pivra_gpu_has_pitch_filter(stream) || quarters < 1 || quarters > PIVRA_RATE_MAX)
    return FALSE;
  PivraGpuMediaFile *self = PIVRA_GPU_MEDIA_FILE(stream);
  if (!self->pipeline || !self->video_pad || self->seeking || self->pending_rate ||
      !gtk_media_stream_is_prepared(stream) || !gtk_media_stream_is_seekable(stream) ||
      gtk_media_stream_get_ended(stream) || gtk_media_stream_get_error(stream))
    return FALSE;
  if (quarters == self->rate)
    return TRUE;
  gint64 position;
  if (!gst_element_query_position(self->pipeline, GST_FORMAT_TIME, &position) || position < 0 ||
      !pivra_rate_seek(self->pipeline, quarters, position))
    return FALSE;
  self->pending_rate = quarters;
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_RATE_PENDING]);
  return TRUE;
}

static void finish_rate_change(PivraGpuMediaFile *self)
{
  guint requested = self->pending_rate;
  self->pending_rate = 0;
  /* The seek request itself is not a confirmation. Check the new video
   * segment after preroll, before showing a new rate in the window. */
  if (pivra_rate_in_segment(self->video_pad, requested)) {
    self->rate = requested;
    g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_RATE]);
  } else {
    g_printerr("Pivra: GStreamer did not confirm the requested playback speed\n");
  }
  if (gtk_media_stream_is_seeking(GTK_MEDIA_STREAM(self)))
    seek(GTK_MEDIA_STREAM(self), self->seek_target);
  g_object_notify_by_pspec(G_OBJECT(self), properties[PROP_RATE_PENDING]);
}

static gboolean bus_message(GstBus *bus,GstMessage *message,gpointer data) {
  (void)bus;
  PivraGpuMediaFile *self=g_weak_ref_get(data);
  if(!self) return G_SOURCE_REMOVE;
  switch(GST_MESSAGE_TYPE(message)) {
    case GST_MESSAGE_STREAM_COLLECTION: {
      GstStreamCollection *collection=NULL;gst_message_parse_stream_collection(message,&collection);
      self->audio=FALSE;self->video=FALSE;
      for(guint i=0;collection && i<gst_stream_collection_get_size(collection);i++) {
        GstStreamType type=gst_stream_get_stream_type(gst_stream_collection_get_stream(collection,i));
        self->audio |= (type&GST_STREAM_TYPE_AUDIO)!=0;self->video |= (type&GST_STREAM_TYPE_VIDEO)!=0;
      }
      gst_clear_object(&collection);break;
    }
    case GST_MESSAGE_ASYNC_DONE: {
      if (GST_MESSAGE_SRC(message) != GST_OBJECT(self->pipeline))
        break;
      /* Preroll preparation may initiate resume seeking reentrantly. */
      gboolean was_seeking=self->seeking;
      gboolean rate_pending=self->pending_rate != 0;
      prepared(self);update(self);
      if(rate_pending && self->pending_rate && self->video_pad)
        finish_rate_change(self);
      else if(was_seeking && self->seeking && self->pipeline) {
        self->seeking=FALSE;
        if(self->seek_target!=self->active_target) seek(GTK_MEDIA_STREAM(self),self->seek_target);
        else gtk_media_stream_seek_success(GTK_MEDIA_STREAM(self));
      }
      break;
    }
    case GST_MESSAGE_EOS:
      if(self->pipeline && gtk_media_stream_get_loop(GTK_MEDIA_STREAM(self))) {
        pivra_rate_seek(self->pipeline,self->rate,0);
        gst_element_set_state(self->pipeline,GST_STATE_PLAYING);
      } else gtk_media_stream_stream_ended(GTK_MEDIA_STREAM(self));
      break;
    case GST_MESSAGE_ERROR: failure(self,message);break;
    default: break;
  }
  g_object_unref(self);return G_SOURCE_CONTINUE;
}
static void audio(GtkMediaStream *stream,gboolean muted,double volume) {
  PivraGpuMediaFile *self=PIVRA_GPU_MEDIA_FILE(stream);
  if(self->pipeline) g_object_set(self->pipeline,"mute",muted,"volume",volume*volume*volume,NULL);
}
static void open_file(GtkMediaFile *file) {
  PivraGpuMediaFile *self=PIVRA_GPU_MEDIA_FILE(file);close_file(file);
  GFile *source=gtk_media_file_get_file(file);
  if(!source) { gtk_media_stream_error(GTK_MEDIA_STREAM(self),G_IO_ERROR,G_IO_ERROR_NOT_SUPPORTED,"GPU video requires a file URI");return; }
  self->pipeline=gst_element_factory_make("playbin3",NULL);
  if(!self->pipeline) { gtk_media_stream_error(GTK_MEDIA_STREAM(self),G_IO_ERROR,G_IO_ERROR_NOT_SUPPORTED,"Missing GStreamer playbin3");return; }
  gst_object_ref_sink(self->pipeline);
  g_object_weak_ref(G_OBJECT(self->pipeline),pipeline_retired,NULL);
  GError *sink_error=NULL;
  GstElement *sink=pivra_gpu_paintable_create_sink_full(PIVRA_GPU_PAINTABLE(self->paintable), &sink_error);
  /* A NULL sink must never select playbin's automatic/CPU/other-window sink. */
  if(!sink) { close_file(file);gtk_media_stream_gerror(GTK_MEDIA_STREAM(self),sink_error);return; }
  gst_object_ref_sink(sink);
  self->video_pad = gst_element_get_static_pad(sink, "sink");
  gchar *uri=g_file_get_uri(source);g_object_set(self->pipeline,"uri",uri,"video-sink",sink,NULL);
  g_free(uri);gst_object_unref(sink);
  GstElement *tempo = gst_element_factory_make("scaletempo", NULL);
  self->pitch_filter = tempo != NULL;
  if (tempo) {
    gst_object_ref_sink(tempo);
    g_object_set(self->pipeline, "audio-filter", tempo, NULL);
    gst_object_unref(tempo);
  }
  GstBus *bus=gst_element_get_bus(self->pipeline);
  self->bus_watch=gst_bus_add_watch_full(bus,G_PRIORITY_DEFAULT,bus_message,weak_new(self),weak_free);gst_object_unref(bus);
  GSource *timer=g_timeout_source_new(100);g_source_set_callback(timer,tick,weak_new(self),weak_free);
  self->tick=g_source_attach(timer,g_main_context_default());g_source_unref(timer);
  self->video=TRUE;self->audio=FALSE;self->seeking=FALSE;
  audio(GTK_MEDIA_STREAM(self),gtk_media_stream_get_muted(GTK_MEDIA_STREAM(self)),gtk_media_stream_get_volume(GTK_MEDIA_STREAM(self)));
  if(gst_element_set_state(self->pipeline,GST_STATE_PAUSED)==GST_STATE_CHANGE_FAILURE) {
    GstBus *failed_bus=gst_element_get_bus(self->pipeline);GstMessage *error=gst_bus_pop_filtered(failed_bus,GST_MESSAGE_ERROR);
    if(error) { failure(self,error);gst_message_unref(error); }
    else if(!gtk_media_stream_get_error(GTK_MEDIA_STREAM(self))) gtk_media_stream_error(GTK_MEDIA_STREAM(self),G_IO_ERROR,G_IO_ERROR_FAILED,"GPU pipeline failed to preroll");
    gst_object_unref(failed_bus);
  }
}
static void close_file(GtkMediaFile *file) {
  PivraGpuMediaFile *self=PIVRA_GPU_MEDIA_FILE(file);
  pivra_video_lifecycle("stream-close-begin", self);
  if(self->tick) { g_source_remove(self->tick);self->tick=0; }
  if(self->bus_watch) { g_source_remove(self->bus_watch);self->bus_watch=0; }
  GstElement *pipeline=self->pipeline;self->pipeline=NULL;self->seeking=FALSE;
  self->rate = PIVRA_RATE_NORMAL;
  self->pending_rate = 0;
  self->pitch_filter = FALSE;
  gst_clear_object(&self->video_pad);
  if(pipeline) {
    gst_element_set_state(pipeline,GST_STATE_NULL);
    if(g_getenv("PIVRA_TRACE_VIDEO")) g_printerr("PIVRA_GPU_CLOSE pipeline=%p refs=%d\n",(void *)pipeline,GST_OBJECT_REFCOUNT_VALUE(pipeline));
    gst_object_unref(pipeline);
  }
  if(self->paintable) pivra_gpu_paintable_reset(PIVRA_GPU_PAINTABLE(self->paintable));
  pivra_video_lifecycle("stream-close-end", self);
}
static gboolean play(GtkMediaStream *stream) {
  PivraGpuMediaFile *self=PIVRA_GPU_MEDIA_FILE(stream);
  return self->pipeline && gst_element_set_state(self->pipeline,GST_STATE_PLAYING)!=GST_STATE_CHANGE_FAILURE;
}
static void pause_stream(GtkMediaStream *stream) { PivraGpuMediaFile *self=PIVRA_GPU_MEDIA_FILE(stream);if(self->pipeline) gst_element_set_state(self->pipeline,GST_STATE_PAUSED); }
static void seek(GtkMediaStream *stream,gint64 timestamp) {
  PivraGpuMediaFile *self=PIVRA_GPU_MEDIA_FILE(stream);self->seek_target=timestamp;
  if(self->seeking || self->pending_rate) return;
  self->active_target=timestamp;
  self->seeking=self->pipeline && timestamp>=0 && timestamp<=G_MAXINT64/GST_USECOND &&
    pivra_rate_seek(self->pipeline,self->rate,timestamp*GST_USECOND);
  if(!self->seeking) gtk_media_stream_seek_failed(stream);
}
static void realize(GtkMediaStream *s,GdkSurface *surface) { pivra_gpu_paintable_realize(PIVRA_GPU_PAINTABLE(PIVRA_GPU_MEDIA_FILE(s)->paintable),surface); }
static void unrealize(GtkMediaStream *s,GdkSurface *surface) { pivra_gpu_paintable_unrealize(PIVRA_GPU_PAINTABLE(PIVRA_GPU_MEDIA_FILE(s)->paintable),surface); }
static void dispose(GObject *object) {
  PivraGpuMediaFile *self=PIVRA_GPU_MEDIA_FILE(object);close_file(GTK_MEDIA_FILE(self));
  if(self->paintable) { g_signal_handlers_disconnect_by_data(self->paintable,self);g_clear_object(&self->paintable); }
  G_OBJECT_CLASS(pivra_gpu_media_file_parent_class)->dispose(object);
}
static void get_property(GObject *object, guint id, GValue *value, GParamSpec *pspec)
{
  PivraGpuMediaFile *self = PIVRA_GPU_MEDIA_FILE(object);
  switch (id) {
    case PROP_RATE: g_value_set_uint(value, self->rate); break;
    case PROP_RATE_PENDING: g_value_set_boolean(value, self->pending_rate != 0); break;
    default: G_OBJECT_WARN_INVALID_PROPERTY_ID(object, id, pspec);
  }
}

static void pivra_gpu_media_file_class_init(PivraGpuMediaFileClass *klass) {
  GtkMediaFileClass *file=GTK_MEDIA_FILE_CLASS(klass);file->open=open_file;file->close=close_file;
  GtkMediaStreamClass *stream=GTK_MEDIA_STREAM_CLASS(klass);stream->play=play;stream->pause=pause_stream;
  stream->seek=seek;stream->update_audio=audio;stream->realize=realize;stream->unrealize=unrealize;
  G_OBJECT_CLASS(klass)->dispose=dispose;
  G_OBJECT_CLASS(klass)->get_property=get_property;
  properties[PROP_RATE] = g_param_spec_uint("playback-rate-quarters", NULL, NULL, 1,
                                            PIVRA_RATE_MAX, PIVRA_RATE_NORMAL, G_PARAM_READABLE);
  properties[PROP_RATE_PENDING] = g_param_spec_boolean("playback-rate-pending", NULL, NULL,
                                                       FALSE, G_PARAM_READABLE);
  g_object_class_install_properties(G_OBJECT_CLASS(klass), N_PROPERTIES, properties);
}
static void pivra_gpu_media_file_init(PivraGpuMediaFile *self) {
  self->rate = PIVRA_RATE_NORMAL;
  self->paintable=pivra_gpu_paintable_new();
  g_signal_connect_swapped(self->paintable,"invalidate-size",G_CALLBACK(gdk_paintable_invalidate_size),self);
  g_signal_connect_swapped(self->paintable,"invalidate-contents",G_CALLBACK(gdk_paintable_invalidate_contents),self);
}
