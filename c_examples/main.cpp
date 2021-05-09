#include "../bufro.h"
#include <GLFW/glfw3.h>
#include <iostream>
#include <cmath>

Renderer* surface;

void framebuffer_size_callback(GLFWwindow* window, int width, int height);
void processInput(GLFWwindow *window);

// settings
const unsigned int SCR_WIDTH = 800;
const unsigned int SCR_HEIGHT = 600;

const void* load_ptrs(const char* s) {
    GLFWglproc ptr = glfwGetProcAddress(s);
    if (ptr == NULL) {
        printf("Failure to load %s\n", s);
    }
    return reinterpret_cast<const void*>(glfwGetProcAddress(s));
}

int main()
{
    // glfw: initialize and configure
    // ------------------------------
    glfwInit();
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 1);
    glfwWindowHint(GLFW_SAMPLES, 4);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

#ifdef __APPLE__
    glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT, GL_TRUE);
#endif

    // glfw window creation
    // --------------------
    GLFWwindow* window = glfwCreateWindow(SCR_WIDTH, SCR_HEIGHT, "LearnOpenGL", NULL, NULL);
    if (window == NULL)
    {
        std::cout << "Failed to create GLFW window" << std::endl;
        glfwTerminate();
        return -1;
    }
    glfwMakeContextCurrent(window);
    glfwSetFramebufferSizeCallback(window, framebuffer_size_callback);



    surface = bfr_create_surface(load_ptrs);

    // render loop
    // -----------
    bfr_set_clear_color(surface, bfr_colorf(0.5, 0.5, 0.5, 0.5));
    int x = 0;
    while (!glfwWindowShouldClose(window))
    {
        // input
        // -----
        processInput(window);

//                        ctx.circle(x, y, 60., bufro::Color::from_8(191, 134, 53, 1));
//                        ctx.circle(x, y, 50., bufro::Color::from_8(255, 179, 71, 1));
        bfr_rect(surface, x, 300, 100, 100, bfr_color8(100, 100, 100, 1));
        bfr_circle(surface, x, 100, 100, bfr_color8(191, 134, 53, 1));
        bfr_circle(surface, x, 100, 90, bfr_color8(255, 179, 71, 1));
        x++;

        // glfw: swap buffers and poll IO events (keys pressed/released, mouse moved etc.)
        // -------------------------------------------------------------------------------
        bfr_flush(surface);
        glfwSwapBuffers(window);
        glfwPollEvents();
    }

    // glfw: terminate, clearing all previously allocated GLFW resources.
    // ------------------------------------------------------------------
    glfwTerminate();
    return 0;
}

// process all input: query GLFW whether relevant keys are pressed/released this frame and react accordingly
// ---------------------------------------------------------------------------------------------------------
void processInput(GLFWwindow *window)
{
    if(glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS)
        glfwSetWindowShouldClose(window, true);
}

// glfw: whenever the window size changed (by OS or user resize) this callback function executes
// ---------------------------------------------------------------------------------------------
void framebuffer_size_callback(GLFWwindow* window, int width, int height)
{
    // make sure the viewport matches the new window dimensions; note that width and 
    // height will be significantly larger than specified on retina displays.
    bfr_resize(surface, width, height);
}