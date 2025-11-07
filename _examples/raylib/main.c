#include "raylib.h"
#include "nyx.h"
#include <stdbool.h>

Color read_color(Vm *vm, size_t addr) {
    Color color = (Color){
        .r = vm_mem_read_byte(vm, addr),
        .g = vm_mem_read_byte(vm, addr+1),
        .b = vm_mem_read_byte(vm, addr+2),
        .a = vm_mem_read_byte(vm, addr+3),
    };
    return color;
}

int init_window(Vm *vm) {
    int width = vm_get_reg_int(vm, REG_D0);
    int height = vm_get_reg_int(vm, REG_D1);
    const char *title = vm_mem_read_cstr(vm, vm_get_reg_int(vm, REG_Q2));
    InitWindow(width, height, title);
    return 0;
}

int close_window(Vm *vm) {
    CloseWindow();
    return 0;
}

int set_target_fps(Vm *vm) {
    int fps = vm_get_reg_int(vm, REG_D0);
    SetTargetFPS(fps);
    return 0;
}

int window_should_close(Vm *vm) {
    vm_set_reg_int(vm, REG_B0, WindowShouldClose());
    return 0;
}

int begin_drawing(Vm *vm) {
    BeginDrawing();
    return 0;
}

int end_drawing(Vm *vm) {
    EndDrawing();
    return 0;
}

int clear_background(Vm *vm) {
    Color color = read_color(vm, vm_get_reg_int(vm, REG_Q0));
    ClearBackground(color);
    return 0;
}

int draw_text(Vm *vm) {
    const char *text = vm_mem_read_cstr(vm, vm_get_reg_int(vm, REG_Q0));
    int x = vm_get_reg_int(vm, REG_D1);
    int y = vm_get_reg_int(vm, REG_D2);
    int font_size = vm_get_reg_int(vm, REG_D3);
    Color color = read_color(vm, vm_get_reg_int(vm, REG_Q4));
    DrawText(text, x, y, font_size, color);
    return 0;
}
