addi x1 x0 100
addi a3 x1 7
lh x1 0(x0)

addi a0 x0 8

addi t1 x0 0
beq a0 x0 done
addi t0 x0 0
addi t1 x0 1
beq a0 t1 done
addi a0 a0 -2
loop:
add t2 t0 t1
addi t0 t1 0
addi t1 t2 0
beq a0 x0 done
addi a0 a0 -1
jal x0 loop
done:
