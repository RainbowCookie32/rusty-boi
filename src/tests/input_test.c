#include <stdio.h> // include this file for the printf() function
#include <gb/gb.h> // include this file for Game Boy functions

void main(void) {

	printf("GameBoy Input Test\n");
	printf("\nCode based in the \nexample available in gbdev\n");
	
	while(1) {
        switch(joypad()) {
		
		    case J_RIGHT:
			    printf("Pressed Right\n");
			    delay(100);
			    break;
		    case J_LEFT:
                printf("Pressed Left\n");
                delay(100);
			    break;
		    case J_UP:
			    printf("Pressed Up\n");
			    delay(100);
			    break;
		    case J_DOWN:
			    printf("Pressed Down\n");
			    delay(100);
			    break;
		    case J_START:
			    printf("Pressed Start\n");
			    delay(100);
			    break;
		    case J_SELECT:
			    printf("Pressed Select\n");
			    delay(100);
			    break;
		    case J_A:
			    printf("Pressed A\n");
			    delay(100);
			    break;
		    case J_B:
			    printf("Pressed B\n");
			    delay(100);
			    break;			
		    default:
			    break;
		}
	}
}
