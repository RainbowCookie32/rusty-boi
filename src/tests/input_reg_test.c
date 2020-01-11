#include <stdio.h> // include this file for the printf() function
#include <gb/gb.h> // include this file for Game Boy functions
#include <gb/hardware.h>

void main(void) {
    UINT8 reg = P1_REG;
    printf("JOYPAD Value Test\n");
    while(1) {
        printf("JOYPAD Value is %x\n", P1_REG);
        reg = P1_REG;
    }
}
