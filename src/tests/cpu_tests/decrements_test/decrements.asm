; Compiled with rgbds

section "Header", rom0[$100]
start:
 nop 
 jp test_init

section "Test", rom0[$150]
; Test start point
test_init:
 ld a, $FF
; Decrement A from $FF to the underflow
a_loop:
 dec a
 jp nz, a_loop
 ld b, $FF
; Decrement B from $FF to the underflow
b_loop:
 dec b
 jp nz, b_loop
 ld c, $FF
; Decrement C from $FF to the underflow
c_loop:
 dec c
 jp nz, c_loop
 ld d, $FF
; Decrement D from $FF to the underflow
d_loop:
 dec d
 jp nz, d_loop
 ld e, $FF
; Decrement E from $FF to the underflow
e_loop:
 dec e
 jp nz, e_loop
 ld h, $FF
; Decrement H from $FF to the underflow
h_loop:
 dec h
 jp nz, h_loop
 ld l, $FF
; Decrement L from $FF to the underflow
l_loop:
 dec l
 jp nz, l_loop
 stop 

section "Filler", rom0[$3fff]
db 0