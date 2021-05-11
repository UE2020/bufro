#include <bufro.h>
#include <GLFW/glfw3.h>
#include <stdio.h>
#include <math.h>

// Settings
const unsigned int SCR_WIDTH = 800;
const unsigned int SCR_HEIGHT = 600;

Renderer* surface;

void framebuffer_size_callback(GLFWwindow* window, int width, int height) {
    // make sure the viewport matches the new window dimensions; note that width and 
    // height will be significantly larger than specified on retina displays.
    bfr_resize(surface, width, height);
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
    GLFWwindow* window = glfwCreateWindow(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", NULL, NULL);
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
    
    int x = 0;
    float r = 0;
    float counter = 0;

    while (!glfwWindowShouldClose(window)) {
        counter += 0.01;

        process_input(window);
        r += 0.1;
        bfr_rect(surface, 300 + sin(counter) * 600, 300, 100, 100, r, bfr_color8(100, 100, 100, 1));
        bfr_circle(surface, 300 + sin(counter) * 600, 200, 100, bfr_color8(191, 134, 53, 1));
        bfr_circle(surface, 300 + sin(counter) * 600, 200, 90, bfr_color8(255, 179, 71, 1));
        x++;

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        bfr_flush(surface);
        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // glfw: terminate, clearing all previously allocated GLFW resources.
    glfwTerminate();
    return 0;
}
