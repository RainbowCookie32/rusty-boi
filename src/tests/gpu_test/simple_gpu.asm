; Compiled with rgbds

section "Header", rom0[$100]
start:
 nop 
 jp test_init

section "Test", rom0[$150]
test_init:
 ld hl, $8000
 ld de, tile
 ld b, $16
load_loop:
 ld a, [de]
 inc de
 ld [hl+], a
 dec b
 jp nz, load_loop
 ld hl, $9800
 ld [hl], 1
 stop

tile: db $00, $00, $00, $00, $24, $24, $00, $00, $81, $81, $7e, $7e, $00, $00, $00, $00

section "Filler", rom0[$3fff]
db 0