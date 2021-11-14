#define GLFW_EXPOSE_NATIVE_X11

#include <bufro.h>
#include <GLFW/glfw3.h>
#include <GLFW/glfw3native.h>
#include <stdio.h>
#include <math.h>

// Settings
const unsigned int SCR_WIDTH = 800;
const unsigned int SCR_HEIGHT = 600;

Painter* ctx;
int width = SCR_WIDTH;
int height = SCR_HEIGHT;

void framebuffer_size_callback(GLFWwindow* window, int _width, int _height) {
    // make sure the viewport matches the new window dimensions; note that width and 
    // height will be significantly larger than specified on retina displays.
    bfr_painter_resize(ctx, _width, _height);
    width = _width;
    height = _height;
}

int main() {
    // glfw: initialize and configure
    glfwInit();
    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);

#ifdef __APPLE__
    glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
#endif

    // Create glfw window
    GLFWwindow* window = glfwCreateWindow(SCR_WIDTH, SCR_HEIGHT, "Bufro Quickstart", NULL, NULL);
    if (window == NULL) {
        puts("Failed to create GLFW window");
        glfwTerminate();
        return -1;
    }
    glfwMakeContextCurrent(window);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);

    // create bufro surface
    BufroXlibWindow xwin = {
        .display = glfwGetX11Display(),
        .window = glfwGetX11Window(window),
    };
    ctx = bfr_painter_from_xlib_window(xwin, SCR_WIDTH, SCR_HEIGHT);

    // animation variables
    float r1 = 0;
    float r2 = 0;
    float scale_animation = 0;

    while (!glfwWindowShouldClose(window)) {
        bfr_painter_rectangle(ctx, 0, 0, width, height, bfr_colorf(0.2, 0.2, 0.2, 1.0));
        scale_animation += 0.02;
        bfr_painter_scale(ctx, sin(scale_animation) / 4 + 1, sin(scale_animation) / 4 + 1);

        // draw frame
        bfr_painter_translate(ctx, width/2, height/2);
        bfr_painter_rotate(ctx, r1);
        bfr_painter_rectangle(ctx, -50, -50, 100, 100, bfr_coloru8(220, 220, 40, 255));
        bfr_painter_rotate(ctx, r2 - r1);
        bfr_painter_translate(ctx, 200, 0);
        bfr_painter_circle(ctx, 0, 0, 50, bfr_coloru8(30, 90, 200, 255));

        // update animation variables
        r1 += 0.05;
        r2 += -0.075;

        // flush
        //BufroFlushResult err = bfr_painter_flush(ctx);
        //switch 
        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // Clean up
    bfr_painter_free(ctx);
    glfwTerminate();
    return 0;
}
