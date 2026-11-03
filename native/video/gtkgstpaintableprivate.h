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

#pragma once

#include <gdk/gdk.h>
#include <gst/gst.h>
#include <graphene.h>

G_BEGIN_DECLS

#define PIVRA_TYPE_GPU_PAINTABLE (pivra_gpu_paintable_get_type ())

G_DECLARE_FINAL_TYPE (PivraGpuPaintable, pivra_gpu_paintable, PIVRA, GPU_PAINTABLE, GObject)

GdkPaintable *  pivra_gpu_paintable_new                   (void);

void            pivra_gpu_paintable_realize               (PivraGpuPaintable        *self,
                                                         GdkSurface             *surface);
void            pivra_gpu_paintable_unrealize             (PivraGpuPaintable        *self,
                                                         GdkSurface             *surface);
void            pivra_gpu_paintable_queue_set_texture     (PivraGpuPaintable        *self,
                                                         GdkTexture             *texture,
                                                         double                  pixel_aspect_ratio,
                                                         const graphene_rect_t  *viewport,
                                                         int                     orientation);

GstElement *pivra_gpu_paintable_create_sink(PivraGpuPaintable *self);

G_END_DECLS
