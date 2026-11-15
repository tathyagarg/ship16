; This program puts random gibberish on the screen until the user presses a key,
; at which point it will display a blue screen of death (BSOD)

DRAW:
  DRWR r1
  SKP.NE r0, rE
  INCY 1
  INCX 1
  ADD r0, rF
  REM r0, rD
  RET

START:
  MOVC 0, 0
  CLS
  MOV r0, 0
  MOV rF, 1
  MOV rE, 0x3F
  MOV rD, 0x40
  SETI BSOD
  VEC 2
  ENINT 0b100
  EI

RLOOP:
  RAND r1
  CALL DRAW
  JMP RLOOP

BSOD:
  MOV r1, 0x03

BLOOP:
  CALL DRAW
  JMP BLOOP
