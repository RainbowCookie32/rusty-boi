section "Header", rom0[$100]
start:
 nop 
 jp test_init

section "Test", rom0[$150]
; Test start point
test_init:
 ld a, $FF
 ld c, $FF
 scf
 sbc a, c
 stop