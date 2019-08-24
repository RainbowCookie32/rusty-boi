; Compiled with rgbds

section "Header", rom0[$100]
start:
 nop 
 jp test_init

section "Test", rom0[$150]
; Test start point
test_init:
 ld a, 0
; Increment A from 0 to the overflow
a_loop:
 inc a
 jp nz, a_loop
 ld b, 0
; Increment B from 0 to the overflow
b_loop:
 inc b
 jp nz, b_loop
 ld c, 0
; Increment C from 0 to the overflow
c_loop:
 inc c
 jp nz, c_loop
 ld d, 0
; Increment D from 0 to the overflow
d_loop:
 inc d
 jp nz, d_loop
 ld e, 0
; Increment E from 0 to the overflow
e_loop:
 inc e
 jp nz, e_loop
 ld h, 0
; Increment H from 0 to the overflow
h_loop:
 inc h
 jp nz, h_loop
 ld l, 0
; Increment L from 0 to the overflow
l_loop:
 inc l
 jp nz, l_loop
 stop 

section "Filler", rom0[$3fff]
db 0