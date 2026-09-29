#include "widget.h"

#include <stdlib.h>

struct Widget {
  int id;
};

static int open_widgets = 0;

Widget *widget_open(int id) {
  Widget *widget = malloc(sizeof *widget);
  if (!widget) {
    return NULL;
  }
  widget->id = id;
  open_widgets++;
  return widget;
}

int widget_id(const Widget *widget) { return widget->id; }

void widget_close(Widget *widget) {
  if (!widget) {
    return;
  }
  free(widget);
  open_widgets--;
}

int widgets_open(void) { return open_widgets; }
