#include <gb/gb.h>  //Angle brackets check the compiler's include folders
#include "sprite.c" //double quotes check the folder of the code that's being compiled

void main(){
    set_sprite_data(0, 8, testSprite);
    set_sprite_tile(0, 0);
    move_sprite(0, 75, 75);
    SHOW_SPRITES;
}
