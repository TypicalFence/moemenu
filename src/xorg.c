#include <X11/Xlib.h>

int keycode_to_utf8(unsigned int keycode, char *buffer) {
  KeySym keysym = 0x1a2;

  Display* display = XOpenDisplay(":0");

  XIM xim = XOpenIM(display, 0, 0, 0);
  XIC xic = XCreateIC(xim, XNInputStyle, XIMPreeditNothing | XIMStatusNothing, NULL);

  XKeyPressedEvent event;
  event.type = KeyPress;
  event.display = display;
  event.state = 0;
  event.keycode = keycode;

  KeySym ignore;
  Status return_status;
  Xutf8LookupString(xic, &event, buffer, 32, &ignore, &return_status);

  XCloseDisplay(display);
  return return_status;
}
