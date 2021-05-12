#include <bufro.h>
#include <GLFW/glfw3.h>
#include <stdio.h>
#include <math.h>

// Settings
const unsigned int SCR_WIDTH = 800;
const unsigned int SCR_HEIGHT = 600;

Renderer* surface;
int width = SCR_WIDTH;
int height = SCR_HEIGHT;

void framebuffer_size_callback(GLFWwindow* window, int _width, int _height) {
    // make sure the viewport matches the new window dimensions; note that width and 
    // height will be significantly larger than specified on retina displays.
    bfr_resize(surface, _width, _height);
    width = _width;
    height = _height;
}

const void* load_ptrs(const char* s) {
    GLFWglproc ptr = glfwGetProcAddress(s);
    if (ptr == NULL) {
        printf("Failed to load %s\n", s);
    }
    return (const void*) glfwGetProcAddress(s);
}

int main() {
    // glfw: initialize and configure
    glfwInit();
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 1);
    glfwWindowHint(GLFW_SAMPLES, 4);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);
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
    surface = bfr_create_surface(load_ptrs);
    bfr_set_clear_color(surface, bfr_colorf(0.2, 0.2, 0.2, 0.2)); // set the bg color

    while (!glfwWindowShouldClose(window)) {
        // draw frame
        bfr_rect(surface, 50, 50, 100, 100, 0, bfr_color8(220, 220, 40, 100));
        bfr_rect(surface, 75, 75, 100, 100, 0, bfr_color8(30, 90, 200, 100));

        bfr_rect(surface, 225, 225, 100, 100, 0, bfr_color8(30, 90, 200, 100));
        bfr_rect(surface, 200, 200, 100, 100, 0, bfr_color8(220, 220, 40, 100));

        // flush
        bfr_flush(surface);
        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // Clean up
    bfr_destroy(surface);
    glfwTerminate();
    return 0;
}
