.section text

START:
    MOV r0, 0
    MOV r1, 1
    MOV r2, 1000

LOOP:
    ADD r0, r1
    MUL r0, r1
    XOR r0, r1
    ADD r1, r1

    SKP<NE> r1, r2
    JMP RESET

    JMP LOOP

RESET:
    MOV r1, 1
    JMP LOOP
