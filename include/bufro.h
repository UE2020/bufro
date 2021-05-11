#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


/// A basic window-less renderer (though you can always just load the function pointers of a window)
struct Renderer;

/// Represents a color with values from 0-1
struct Color {
    float r;
    float g;
    float b;
    float a;
};


extern "C" {

void bfr_circle(Renderer *renderer, float x, float y, float r, Color color);

Color bfr_color8(uint8_t r, uint8_t g, uint8_t b, uint8_t a);

Color bfr_colorf(float r, float g, float b, float a);

Renderer *bfr_create_surface(const void *(*loader)(const char*));

void bfr_destroy(Renderer *renderer);

void bfr_flush(Renderer *renderer);

void bfr_rect(Renderer *renderer,
              float x,
              float y,
              float width,
              float height,
              float angle,
              Color color);

void bfr_reset(Renderer *renderer);

void bfr_resize(Renderer *renderer, int32_t width, int32_t height);

void bfr_restore(Renderer *renderer);

void bfr_rotate(Renderer *renderer, float x);

void bfr_save(Renderer *renderer);

void bfr_scale(Renderer *renderer, float x, float y);

void bfr_set_clear_color(Renderer *renderer, Color color);

void bfr_translate(Renderer *renderer, float x, float y);

} // extern "C"
