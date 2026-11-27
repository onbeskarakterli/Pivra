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

#include <gtk/gtkmediafile.h>

G_BEGIN_DECLS

#define PIVRA_TYPE_GPU_MEDIA_FILE (pivra_gpu_media_file_get_type ())

G_DECLARE_FINAL_TYPE (PivraGpuMediaFile, pivra_gpu_media_file, PIVRA, GPU_MEDIA_FILE, GtkMediaFile)

gboolean pivra_gpu_has_pitch_filter (GtkMediaStream *stream);
gboolean pivra_gpu_set_playback_rate (GtkMediaStream *stream, guint quarters);

G_END_DECLS
