#pragma once

/*
 * A widget. The struct is defined in widget.c and nowhere else, so callers
 * only ever hold a pointer to one.
 */
typedef struct Widget Widget;

/* Opens a widget with the given id, or returns NULL. */
Widget *widget_open(int id);

/* Returns the widget's id. */
int widget_id(const Widget *widget);

/* Closes a widget. Passing NULL does nothing. */
void widget_close(Widget *widget);

/* Returns how many widgets are currently open. */
int widgets_open(void);
