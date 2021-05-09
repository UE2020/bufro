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

Renderer *bfr_create_surface(const void *(*loader)(const char*));

void bfr_destroy(Renderer *renderer);

void bfr_flush(Renderer *renderer);

void bfr_resize(Renderer *renderer, int32_t width, int32_t height);

} // extern "C"
