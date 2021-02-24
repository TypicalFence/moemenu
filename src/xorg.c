/*
 * This file is part of moemenu.
 * Copyright (C) 2021 fence.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
#include <X11/Xlib.h>

int keycode_to_utf8(unsigned int keycode, unsigned int mask, char *buffer) {
  Display* display = XOpenDisplay(":0");

  XIM xim = XOpenIM(display, 0, 0, 0);
  XIC xic = XCreateIC(xim, XNInputStyle, XIMPreeditNothing | XIMStatusNothing, NULL);

  XKeyPressedEvent event;
  event.type = KeyPress;
  event.display = display;
  event.state = mask;
  event.keycode = keycode;

  KeySym ignore;
  Status return_status;
  Xutf8LookupString(xic, &event, buffer, 32, &ignore, &return_status);
  XDestroyIC(xic);
  XCloseIM(xim);
  XCloseDisplay(display);
  return return_status;
}
