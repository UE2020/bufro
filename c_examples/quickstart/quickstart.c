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

void process_input(GLFWwindow *window) {
    if (glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS)
        glfwSetWindowShouldClose(window, true);
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

    // animation variables
    float r1 = 0;
    float r2 = 0;
    float scale_animation = 0;

    while (!glfwWindowShouldClose(window)) {
        process_input(window);

        scale_animation += 0.02;
        bfr_scale(surface, sin(scale_animation) / 4 + 1, sin(scale_animation) / 4 + 1);

        // draw frame
        bfr_translate(surface, width/2, height/2);
        bfr_rotate(surface, r1);
        bfr_rect(surface, -50, -50, 100, 100, 0, bfr_color8(220, 220, 40, 255));
        bfr_rotate(surface, r2 - r1);
        bfr_translate(surface, 200, 0);
        bfr_circle(surface, 0, 0, 50, bfr_color8(30, 90, 200, 255));

        // update animation variables
        r1 += 0.05;
        r2 += -0.075;

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        bfr_flush(surface);
        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // glfw: terminate, clearing all previously allocated GLFW resources.
    glfwTerminate();
    return 0;
}
