#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>


/**
 * A basic window-less renderer (though you can always just load the function pointers of a window)
 */
typedef struct Renderer Renderer;

/**
 * Represents a color with values from 0-1
 */
typedef struct Color {
    float r;
    float g;
    float b;
    float a;
} Color;

void bfr_circle(struct Renderer *renderer, float x, float y, float r, struct Color color);

struct Color bfr_color8(uint8_t r, uint8_t g, uint8_t b, uint8_t a);

struct Color bfr_colorf(float r, float g, float b, float a);

struct Renderer *bfr_create_surface(const void *(*loader)(const char*));

void bfr_destroy(struct Renderer *renderer);

void bfr_flush(struct Renderer *renderer);

void bfr_rect(struct Renderer *renderer,
              float x,
              float y,
              float width,
              float height,
              float angle,
              struct Color color);

void bfr_reset(struct Renderer *renderer);

void bfr_resize(struct Renderer *renderer, int32_t width, int32_t height);

void bfr_restore(struct Renderer *renderer);

void bfr_rotate(struct Renderer *renderer, float x);

void bfr_save(struct Renderer *renderer);

void bfr_scale(struct Renderer *renderer, float x, float y);

void bfr_set_clear_color(struct Renderer *renderer, struct Color color);

void bfr_translate(struct Renderer *renderer, float x, float y);
